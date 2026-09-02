use std::{
    collections::VecDeque,
    sync::{atomic::{AtomicBool, AtomicU32, Ordering}, Arc},
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use flutter_rust_bridge::{frb, StreamSink};
use ggwave_rs::{default_parameters, ffi, GgWave, ProtocolId, SampleFormat};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::sync::mpsc;

static SINK: Lazy<Mutex<Option<StreamSink<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));
static FREQ_BITS: AtomicU32 = AtomicU32::new(12000.0f32.to_bits());
static LISTENING: AtomicBool = AtomicBool::new(false);
static AUDIO_SERIAL: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn app_proto(id: i32) -> ProtocolId {
    match id {
        // App-facing id 5 intentionally maps to ggwave's ULTRASOUND_FAST.
        5 => ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST,
        1 => ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
        _ => ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
    }
}

fn current_freq() -> f32 {
    f32::from_bits(FREQ_BITS.load(Ordering::Relaxed))
}

fn configure_protocol_frequency() {
    let hz = current_freq().round().clamp(8000.0, 19000.0) as i32;
    unsafe {
        ffi::ggwave_rxProtocolSetFreqStart(
            ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST,
            hz,
        );
        ffi::ggwave_txProtocolSetFreqStart(
            ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST,
            hz,
        );
    }
}

fn codec_for(sample_rate: f32) -> Result<GgWave> {
    let _guard = AUDIO_SERIAL.lock();
    configure_protocol_frequency();
    let mut params = default_parameters();
    params.sampleRateInp = sample_rate;
    params.sampleRateOut = sample_rate;
    params.sampleRate = sample_rate;
    params.sampleFormatInp = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
    params.sampleFormatOut = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
    GgWave::new(params).map_err(|e| anyhow!(e.to_string()))
}

fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn f32_to_bytes(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 4);
    for &sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

fn pre_emphasis(samples: &mut [f32]) {
    // Gentle one-pole high-frequency emphasis + requested 1.8x gain.
    let mut previous = 0.0f32;
    for sample in samples {
        let input = *sample;
        let emphasized = (input - 0.85 * previous) * 1.8;
        *sample = emphasized.clamp(-1.0, 1.0);
        previous = input;
    }
}

#[frb]
pub fn init_rust() -> Result<()> {
    // Force a cheap codec creation now so unsupported native setups fail early.
    let _ = codec_for(48_000.0)?;
    Ok(())
}

#[frb]
pub fn set_ultrasonic_freq(freq_start: f32) -> Result<()> {
    if !(8000.0..=19000.0).contains(&freq_start) {
        bail!("freq_start must be in 8000..=19000 Hz");
    }
    FREQ_BITS.store(freq_start.to_bits(), Ordering::Relaxed);
    let _guard = AUDIO_SERIAL.lock();
    configure_protocol_frequency();
    Ok(())
}

#[frb]
pub fn encode(data: Vec<u8>, protocol_id: i32, volume: i32) -> Result<Vec<f32>> {
    if data.len() > 140 {
        bail!("ggwave payload is limited to 140 bytes by bingo_qr");
    }
    if !(0..=100).contains(&volume) {
        bail!("volume must be 0..=100");
    }

    let gg = codec_for(48_000.0)?;
    let raw = {
        let _guard = AUDIO_SERIAL.lock();
        gg.encode(&data, app_proto(protocol_id), volume)
            .map_err(|e| anyhow!(e.to_string()))?
    };
    let mut waveform = bytes_to_f32(&raw);
    if protocol_id >= 4 {
        pre_emphasis(&mut waveform);
    }
    Ok(waveform)
}

#[frb]
pub fn start_listening(protocol_id: i32) -> Result<()> {
    stop_listening()?;
    LISTENING.store(true, Ordering::SeqCst);

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);
    let protocol = app_proto(protocol_id);

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
            Ok(v) => v,
            Err(_) => {
                LISTENING.store(false, Ordering::SeqCst);
                return;
            }
        };

        // ggwave-rs marks GgWave !Send/!Sync because upstream uses a global
        // unsynchronised instance table. The callback below is additionally
        // serialized with AUDIO_SERIAL so no ggwave operation can run concurrently.
        struct SerializedCodec(GgWave);
        unsafe impl Send for SerializedCodec {}
        let codec = Arc::new(Mutex::new(SerializedCodec(codec)));
        let dedup = Arc::new(Mutex::new(VecDeque::<(Vec<u8>, Instant)>::new()));

        let consume = move |mono: Vec<f32>| {
            if !LISTENING.load(Ordering::Relaxed) {
                return;
            }
            let bytes = f32_to_bytes(&mono);
            let decoded = {
                let _serial = AUDIO_SERIAL.lock();
                let guard = codec.lock();
                guard.0.decode(&bytes).ok().flatten()
            };
            let Some(payload) = decoded else { return; };
            if payload.is_empty() { return; }

            let now = Instant::now();
            let mut window = dedup.lock();
            while window.front().is_some_and(|(_, at)| now.duration_since(*at) > Duration::from_millis(800)) {
                window.pop_front();
            }
            if window.iter().any(|(seen, _)| *seen == payload) {
                return;
            }
            window.push_back((payload.clone(), now));
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
        let _ = protocol;
    });

    thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() else {
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
    let device = host.default_output_device().context("no default output device")?;
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
                            let s = samples.get(*index).copied().unwrap_or(0.0);
                            if *index < samples.len() { *index += 1; }
                            let value: $sample_ty = $convert(s);
                            for ch in frame { *ch = value; }
                        }
                    },
                    err_fn,
                    None,
                )
            }};
        }

        let stream_result = match supported.sample_format() {
            cpal::SampleFormat::F32 => build_stream!(f32, |v: f32| v),
            cpal::SampleFormat::I16 => build_stream!(i16, |v: f32| (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16),
            cpal::SampleFormat::U16 => build_stream!(u16, |v: f32| (((v.clamp(-1.0, 1.0) + 1.0) * 0.5) * u16::MAX as f32) as u16),
            _ => return,
        };
        let Ok(stream) = stream_result else { return; };
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
