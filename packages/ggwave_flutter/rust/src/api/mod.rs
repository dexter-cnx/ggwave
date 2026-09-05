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

pub(crate) const ENCODE_SAMPLE_RATE: f32 = 48_000.0;
pub(crate) const PLAYBACK_TAIL_MS: u64 = 300;

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

fn resample_linear(samples: &[f32], source_rate: f32, target_rate: f32) -> Vec<f32> {
    if samples.is_empty() || source_rate <= 0.0 || target_rate <= 0.0 {
        return samples.to_vec();
    }
    if (source_rate - target_rate).abs() < f32::EPSILON {
        return samples.to_vec();
    }

    let target_len = ((samples.len() as f64) * target_rate as f64 / source_rate as f64)
        .round()
        .max(1.0) as usize;
    let ratio = source_rate as f64 / target_rate as f64;
    let mut out = Vec::with_capacity(target_len);
    for i in 0..target_len {
        let source_pos = i as f64 * ratio;
        let left = source_pos.floor() as usize;
        let right = (left + 1).min(samples.len() - 1);
        let fraction = (source_pos - left as f64) as f32;
        out.push(samples[left] + (samples[right] - samples[left]) * fraction);
    }
    out
}

#[frb]
pub fn init_rust() -> Result<()> {
    eprintln!("[GGWAVE_NATIVE] init_rust: start");
    let _ = codec_for(ENCODE_SAMPLE_RATE).map_err(|error| {
        eprintln!("[GGWAVE_NATIVE][ERROR] init_rust: {error:#}");
        error
    })?;
    eprintln!("[GGWAVE_NATIVE] init_rust: success");
    Ok(())
}

#[frb]
pub fn set_ultrasonic_freq(freq_start: f32) -> Result<()> {
    eprintln!("[GGWAVE_NATIVE] set_ultrasonic_freq: {freq_start} Hz");
    let tuning = Tuning {
        ultrasonic_hz: freq_start,
        ..Tuning::default()
    };
    tuning.validate().map_err(|error| {
        eprintln!("[GGWAVE_NATIVE][ERROR] tuning validation: {error:#}");
        error
    })?;
    FREQ_BITS.store(freq_start.to_bits(), Ordering::Relaxed);

    let _ = codec_for(ENCODE_SAMPLE_RATE).map_err(|error| {
        eprintln!("[GGWAVE_NATIVE][ERROR] tuning codec recreate: {error:#}");
        error
    })?;
    Ok(())
}

#[frb]
pub fn encode(data: Vec<u8>, protocol_id: i32, volume: i32) -> Result<Vec<f32>> {
    eprintln!(
        "[GGWAVE_NATIVE] encode: bytes={} protocol_id={} volume={}",
        data.len(), protocol_id, volume
    );
    let codec = codec_for(ENCODE_SAMPLE_RATE).map_err(|error| {
        eprintln!("[GGWAVE_NATIVE][ERROR] encode codec init: {error:#}");
        error
    })?;
    let protocol = Protocol::from_app_id(protocol_id);
    let waveform = {
        let _guard = CODEC_SERIAL.lock();
        codec.encode(&data, protocol, volume).map_err(|error| {
            eprintln!("[GGWAVE_NATIVE][ERROR] encode: {error:#}");
            error
        })?
    };
    eprintln!("[GGWAVE_NATIVE] encode: samples={}", waveform.len());
    Ok(waveform)
}

#[frb]
pub fn start_listening(_protocol_id: i32) -> Result<()> {
    eprintln!("[GGWAVE_NATIVE] start_listening: requested");
    stop_listening()?;
    LISTENING.store(true, Ordering::SeqCst);

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

    thread::spawn(move || {
        let host = cpal::default_host();
        let Some(device) = host.default_input_device() else {
            eprintln!("[GGWAVE_NATIVE][ERROR] listen: no default input device");
            LISTENING.store(false, Ordering::SeqCst);
            return;
        };
        let supported = match device.default_input_config() {
            Ok(config) => config,
            Err(error) => {
                eprintln!("[GGWAVE_NATIVE][ERROR] listen default_input_config: {error}");
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };
        let config: cpal::StreamConfig = supported.clone().into();
        let channels = config.channels.max(1) as usize;
        let sample_rate = config.sample_rate.0 as f32;
        eprintln!(
            "[GGWAVE_NATIVE] listen config: rate={} channels={} format={:?}",
            sample_rate,
            channels,
            supported.sample_format()
        );
        let codec = match codec_for(sample_rate) {
            Ok(codec) => codec,
            Err(error) => {
                eprintln!("[GGWAVE_NATIVE][ERROR] listen codec init: {error:#}");
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };

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
                match guard.0.decode(&mono) {
                    Ok(decoded) => decoded,
                    Err(error) => {
                        eprintln!("[GGWAVE_NATIVE][ERROR] decode: {error:#}");
                        None
                    }
                }
            };
            let Some(payload) = decoded else {
                return;
            };
            if payload.is_empty() || !deduper.lock().accept(&payload) {
                return;
            }
            eprintln!("[GGWAVE_NATIVE] decoded payload bytes={}", payload.len());
            if let Err(error) = tx.try_send(payload) {
                eprintln!("[GGWAVE_NATIVE][ERROR] rx queue send: {error}");
            }
        };

        let err_fn = |error| {
            eprintln!("[GGWAVE_NATIVE][ERROR] input stream callback: {error}");
        };
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
            other => {
                eprintln!("[GGWAVE_NATIVE][ERROR] unsupported input sample format: {other:?}");
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };

        let stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("[GGWAVE_NATIVE][ERROR] build input stream: {error}");
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };
        if let Err(error) = stream.play() {
            eprintln!("[GGWAVE_NATIVE][ERROR] input stream play: {error}");
            LISTENING.store(false, Ordering::SeqCst);
            return;
        }
        eprintln!("[GGWAVE_NATIVE] listening: active");
        while LISTENING.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }
        drop(stream);
        eprintln!("[GGWAVE_NATIVE] listening: stopped");
    });

    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("[GGWAVE_NATIVE][ERROR] tokio runtime: {error}");
                return;
            }
        };
        runtime.block_on(async move {
            while let Some(payload) = rx.recv().await {
                if let Some(sink) = SINK.lock().as_ref() {
                    if let Err(error) = sink.add(payload) {
                        eprintln!("[GGWAVE_NATIVE][ERROR] FRB sink add: {error:?}");
                    }
                } else {
                    eprintln!("[GGWAVE_NATIVE][ERROR] FRB message sink is not registered");
                }
            }
        });
    });

    Ok(())
}

#[frb]
pub fn stop_listening() -> Result<()> {
    LISTENING.store(false, Ordering::SeqCst);
    eprintln!("[GGWAVE_NATIVE] stop_listening");
    Ok(())
}

#[frb]
pub fn play_waveform(waveform: Vec<f32>) -> Result<()> {
    eprintln!("[GGWAVE_NATIVE] play_waveform: samples={}", waveform.len());
    if waveform.is_empty() {
        eprintln!("[GGWAVE_NATIVE][ERROR] play_waveform: waveform is empty");
        bail!("waveform is empty");
    }
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("no default output device")
        .map_err(|error| {
            eprintln!("[GGWAVE_NATIVE][ERROR] output device: {error:#}");
            error
        })?;
    let supported = device.default_output_config().map_err(|error| {
        eprintln!("[GGWAVE_NATIVE][ERROR] default_output_config: {error}");
        error
    })?;
    let config: cpal::StreamConfig = supported.clone().into();
    let channels = config.channels.max(1) as usize;
    let output_rate = config.sample_rate.0 as f32;
    let waveform = resample_linear(&waveform, ENCODE_SAMPLE_RATE, output_rate);
    let playback_ms = ((waveform.len() as f64 / output_rate as f64) * 1000.0).ceil() as u64
        + PLAYBACK_TAIL_MS;
    eprintln!(
        "[GGWAVE_NATIVE] output config: rate={} channels={} format={:?} samples={} playback_ms={}",
        config.sample_rate.0,
        channels,
        supported.sample_format(),
        waveform.len(),
        playback_ms
    );

    thread::spawn(move || {
        let cursor = Arc::new(Mutex::new(0usize));
        let samples = Arc::new(waveform);
        let err_fn = |error| {
            eprintln!("[GGWAVE_NATIVE][ERROR] output stream callback: {error}");
        };

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
            other => {
                eprintln!("[GGWAVE_NATIVE][ERROR] unsupported output sample format: {other:?}");
                return;
            }
        };
        let stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("[GGWAVE_NATIVE][ERROR] build output stream: {error}");
                return;
            }
        };
        match stream.play() {
            Ok(()) => {
                eprintln!("[GGWAVE_NATIVE] output stream: playing for {playback_ms} ms");
                thread::sleep(Duration::from_millis(playback_ms));
            }
            Err(error) => {
                eprintln!("[GGWAVE_NATIVE][ERROR] output stream play: {error}");
            }
        }
        drop(stream);
        eprintln!("[GGWAVE_NATIVE] output stream: finished");
    });
    Ok(())
}

#[frb]
pub fn create_on_message_stream(sink: StreamSink<Vec<u8>>) {
    eprintln!("[GGWAVE_NATIVE] FRB message sink registered");
    *SINK.lock() = Some(sink);
}
