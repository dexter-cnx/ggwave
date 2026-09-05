use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use flutter_rust_bridge::frb;
use ggwave_core::{Codec, PacketDeduper, Protocol, Tuning};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::frb_generated::StreamSink;

static SINK: Lazy<Mutex<Option<StreamSink<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));
static FREQ_BITS: AtomicU32 = AtomicU32::new(12_000.0f32.to_bits());
static LISTENING: AtomicBool = AtomicBool::new(false);

// ggwave upstream uses process-global native state and its Codec is intentionally
// !Send/!Sync. Serialize every codec operation in this adapter.
static CODEC_SERIAL: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn current_tuning() -> Tuning {
    Tuning {
        ultrasonic_hz: f32::from_bits(FREQ_BITS.load(Ordering::Relaxed)),
        ..Tuning::default()
    }
}

fn codec_for(sample_rate: f32) -> Result<Codec> {
    let _guard = CODEC_SERIAL.lock();
    Ok(Codec::with_sample_rate(current_tuning(), sample_rate)?)
}

#[frb]
pub fn init_rust() -> Result<()> {
    // Fail early if the native codec cannot be created on this platform.
    let _ = codec_for(48_000.0)?;
    Ok(())
}

#[frb]
pub fn set_ultrasonic_freq(freq_start: f32) -> Result<()> {
    let tuning = Tuning {
        ultrasonic_hz: freq_start,
        ..Tuning::default()
    };
    tuning.validate()?;
    FREQ_BITS.store(freq_start.to_bits(), Ordering::Relaxed);

    // Recreate once so the upstream protocol globals are updated immediately.
    let _ = codec_for(48_000.0)?;
    Ok(())
}

#[frb]
pub fn encode(data: Vec<u8>, protocol_id: i32, volume: i32) -> Result<Vec<f32>> {
    let codec = codec_for(48_000.0)?;
    let protocol = Protocol::from_app_id(protocol_id);
    let waveform = {
        let _guard = CODEC_SERIAL.lock();
        codec.encode(&data, protocol, volume)?
    };
    Ok(waveform)
}

#[frb]
pub fn start_listening(_protocol_id: i32) -> Result<()> {
    stop_listening()?;
    LISTENING.store(true, Ordering::SeqCst);

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

    thread::spawn(move || {
        let host = cpal::default_host();
        let Some(device) = host.default_input_device() else {
            LISTENING.store(false, Ordering::SeqCst);
            return;
        };
        let Ok(supported) = device.default_input_config() else {
            LISTENING.store(false, Ordering::SeqCst);
            return;
        };
        let config: cpal::StreamConfig = supported.clone().into();
        let channels = config.channels.max(1) as usize;
        let sample_rate = config.sample_rate.0 as f32;
        let codec = match codec_for(sample_rate) {
            Ok(codec) => codec,
            Err(_) => {
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };

        // CPAL callbacks may execute on another native audio thread. The wrapper
        // is safe here because all accesses are serialized through CODEC_SERIAL.
        struct SerializedCodec(Codec);
        unsafe impl Send for SerializedCodec {}
        let codec = Arc::new(Mutex::new(SerializedCodec(codec)));
        let deduper = Arc::new(Mutex::new(PacketDeduper::default()));

        let consume = move |mono: Vec<f32>| {
            if !LISTENING.load(Ordering::Relaxed) {
                return;
            }
            let decoded = {
                let _serial = CODEC_SERIAL.lock();
                let guard = codec.lock();
                guard.0.decode(&mono).ok().flatten()
            };
            let Some(payload) = decoded else {
                return;
            };
            if payload.is_empty() || !deduper.lock().accept(&payload) {
                return;
            }
            let _ = tx.try_send(payload);
        };

        let err_fn = |_err| {};
        let stream_result = match supported.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mono = data.chunks(channels).map(|frame| frame[0]).collect();
                    consume(mono);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let mono = data
                        .chunks(channels)
                        .map(|frame| frame[0] as f32 / i16::MAX as f32)
                        .collect();
                    consume(mono);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let mono = data
                        .chunks(channels)
                        .map(|frame| (frame[0] as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    consume(mono);
                },
                err_fn,
                None,
            ),
            _ => {
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };

        let Ok(stream) = stream_result else {
            LISTENING.store(false, Ordering::SeqCst);
            return;
        };
        if stream.play().is_err() {
            LISTENING.store(false, Ordering::SeqCst);
            return;
        }
        while LISTENING.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }
        drop(stream);
    });

    thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        runtime.block_on(async move {
            while let Some(payload) = rx.recv().await {
                if let Some(sink) = SINK.lock().as_ref() {
                    let _ = sink.add(payload);
                }
            }
        });
    });

    Ok(())
}

#[frb]
pub fn stop_listening() -> Result<()> {
    LISTENING.store(false, Ordering::SeqCst);
    Ok(())
}

#[frb]
pub fn play_waveform(waveform: Vec<f32>) -> Result<()> {
    if waveform.is_empty() {
        bail!("waveform is empty");
    }
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("no default output device")?;
    let supported = device.default_output_config()?;
    let config: cpal::StreamConfig = supported.clone().into();
    let channels = config.channels.max(1) as usize;

    thread::spawn(move || {
        let cursor = Arc::new(Mutex::new(0usize));
        let samples = Arc::new(waveform);
        let err_fn = |_err| {};

        macro_rules! build_stream {
            ($sample_ty:ty, $convert:expr) => {{
                let cursor = Arc::clone(&cursor);
                let samples = Arc::clone(&samples);
                device.build_output_stream(
                    &config,
                    move |out: &mut [$sample_ty], _| {
                        let mut index = cursor.lock();
                        for frame in out.chunks_mut(channels) {
                            let sample = samples.get(*index).copied().unwrap_or(0.0);
                            if *index < samples.len() {
                                *index += 1;
                            }
                            let value: $sample_ty = $convert(sample);
                            for channel in frame {
                                *channel = value;
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }};
        }

        let stream_result = match supported.sample_format() {
            cpal::SampleFormat::F32 => build_stream!(f32, |v: f32| v),
            cpal::SampleFormat::I16 => build_stream!(i16, |v: f32| {
                (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
            }),
            cpal::SampleFormat::U16 => build_stream!(u16, |v: f32| {
                (((v.clamp(-1.0, 1.0) + 1.0) * 0.5) * u16::MAX as f32) as u16
            }),
            _ => return,
        };
        let Ok(stream) = stream_result else {
            return;
        };
        if stream.play().is_ok() {
            thread::sleep(Duration::from_millis(2500));
        }
        drop(stream);
    });
    Ok(())
}

#[frb]
pub fn create_on_message_stream(sink: StreamSink<Vec<u8>>) {
    *SINK.lock() = Some(sink);
}
