#![cfg(target_os = "android")]

use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{sync_channel, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};

use anyhow::{anyhow, Result};
use ggwave_core::{Codec, PacketDeduper, Protocol};
use oboe::{
    AudioInputCallback, AudioInputStreamSafe, AudioStream, AudioStreamBase, AudioStreamBuilder,
    AudioStreamSafe, DataCallbackResult, Error as OboeError, InputPreset, Mono, PerformanceMode,
    SampleRateConversionQuality, SharingMode,
};

use super::{current_tuning, CODEC_SERIAL, ENCODE_SAMPLE_RATE, LISTENING, SINK};

const FRAMES_PER_CALLBACK: usize = 1024;
const AUDIO_QUEUE_DEPTH: usize = 8;

struct InputCallback {
    tx: SyncSender<[f32; FRAMES_PER_CALLBACK]>,
    callback_count: AtomicU32,
}

impl AudioInputCallback for InputCallback {
    type FrameType = (f32, Mono);

    fn on_audio_ready(
        &mut self,
        _audio_stream: &mut dyn AudioInputStreamSafe,
        audio_data: &[f32],
    ) -> DataCallbackResult {
        if !LISTENING.load(Ordering::Relaxed) {
            return DataCallbackResult::Stop;
        }

        let callback_index = self.callback_count.fetch_add(1, Ordering::Relaxed) + 1;
        if callback_index == 1 || callback_index % 200 == 0 {
            let peak = audio_data
                .iter()
                .fold(0.0f32, |acc, sample| acc.max(sample.abs()));
            eprintln!(
                "[GGWAVE_NATIVE] Oboe input activity: callback={} samples={} peak={:.4}",
                callback_index,
                audio_data.len(),
                peak
            );
        }

        if audio_data.len() == FRAMES_PER_CALLBACK {
            let mut block = [0.0f32; FRAMES_PER_CALLBACK];
            block.copy_from_slice(audio_data);
            if let Err(error) = self.tx.try_send(block) {
                if matches!(error, TrySendError::Full(_)) {
                    eprintln!("[GGWAVE_NATIVE][WARN] Oboe input queue full; dropping one block");
                }
            }
        } else {
            eprintln!(
                "[GGWAVE_NATIVE][WARN] Oboe callback frames={} expected={}",
                audio_data.len(),
                FRAMES_PER_CALLBACK
            );
        }

        DataCallbackResult::Continue
    }

    fn on_error_after_close(
        &mut self,
        _audio_stream: &mut dyn AudioInputStreamSafe,
        error: OboeError,
    ) {
        eprintln!("[GGWAVE_NATIVE][ERROR] Oboe input stream closed: {error:?}");
        LISTENING.store(false, Ordering::SeqCst);
    }
}

fn run_decoder(
    rx: Receiver<[f32; FRAMES_PER_CALLBACK]>,
    sample_rate: f32,
    protocol: Protocol,
) {
    let codec = {
        let _serial = CODEC_SERIAL.lock();
        Codec::with_sample_rate_and_rx_protocol(current_tuning(), sample_rate, protocol)
    };
    let codec = match codec {
        Ok(codec) => codec,
        Err(error) => {
            eprintln!("[GGWAVE_NATIVE][ERROR] Oboe decoder init: {error:#}");
            LISTENING.store(false, Ordering::SeqCst);
            return;
        }
    };
    let mut deduper = PacketDeduper::default();

    eprintln!("[GGWAVE_NATIVE] Oboe RX protocol filter: {protocol:?}");

    while LISTENING.load(Ordering::Relaxed) {
        let block = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(block) => block,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };

        let decoded = {
            let _serial = CODEC_SERIAL.lock();
            match codec.decode(&block) {
                Ok(decoded) => decoded,
                Err(error) => {
                    eprintln!("[GGWAVE_NATIVE][ERROR] Oboe decode: {error:#}");
                    None
                }
            }
        };

        let Some(payload) = decoded else {
            continue;
        };
        if payload.is_empty() || !deduper.accept(&payload) {
            continue;
        }

        eprintln!(
            "[GGWAVE_NATIVE] Oboe decoded payload bytes={}",
            payload.len()
        );
        if let Some(sink) = SINK.lock().as_ref() {
            if let Err(error) = sink.add(payload) {
                eprintln!("[GGWAVE_NATIVE][ERROR] FRB sink add: {error:?}");
            }
        } else {
            eprintln!("[GGWAVE_NATIVE][ERROR] FRB message sink is not registered");
        }
    }

    eprintln!("[GGWAVE_NATIVE] Oboe decoder worker stopped");
}

pub(super) fn start_listening(protocol_id: i32) -> Result<()> {
    let protocol = Protocol::from_app_id(protocol_id);
    eprintln!(
        "[GGWAVE_NATIVE] Android Oboe listen: requested protocol_id={} protocol={:?} operating_rate={} frames_per_callback={}",
        protocol_id,
        protocol,
        ENCODE_SAMPLE_RATE,
        FRAMES_PER_CALLBACK
    );

    LISTENING.store(false, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(60));
    LISTENING.store(true, Ordering::SeqCst);

    let (startup_tx, startup_rx) = sync_channel::<Result<String, String>>(1);

    thread::spawn(move || {
        let (audio_tx, audio_rx) = sync_channel::<[f32; FRAMES_PER_CALLBACK]>(AUDIO_QUEUE_DEPTH);
        let callback = InputCallback {
            tx: audio_tx,
            callback_count: AtomicU32::new(0),
        };

        let builder = AudioStreamBuilder::default()
            .set_input()
            .set_performance_mode(PerformanceMode::LowLatency)
            .set_sharing_mode(SharingMode::Exclusive)
            .set_format::<f32>()
            .set_channel_count::<Mono>()
            .set_sample_rate(ENCODE_SAMPLE_RATE as i32)
            .set_sample_rate_conversion_quality(SampleRateConversionQuality::None)
            .set_input_preset(InputPreset::Unprocessed)
            .set_frames_per_callback(FRAMES_PER_CALLBACK as i32)
            .set_callback(callback);

        let mut stream = match builder.open_stream() {
            Ok(stream) => stream,
            Err(error) => {
                let message = format!("Oboe open input failed: {error:?}");
                eprintln!("[GGWAVE_NATIVE][ERROR] {message}");
                LISTENING.store(false, Ordering::SeqCst);
                let _ = startup_tx.send(Err(message));
                return;
            }
        };

        let actual_rate = stream.get_sample_rate() as f32;
        let actual_frames = stream.get_frames_per_callback();
        let actual_preset = stream.get_input_preset();
        let actual_api = stream.get_audio_api();

        let decoder = thread::spawn(move || run_decoder(audio_rx, actual_rate, protocol));

        if let Err(error) = stream.start() {
            let message = format!("Oboe start input failed: {error:?}");
            eprintln!("[GGWAVE_NATIVE][ERROR] {message}");
            LISTENING.store(false, Ordering::SeqCst);
            let _ = startup_tx.send(Err(message));
            let _ = decoder.join();
            return;
        }

        let details = format!(
            "api={actual_api:?} rate={} channels=mono format=f32 preset={actual_preset:?} frames_per_callback={actual_frames}",
            actual_rate as i32
        );
        eprintln!("[GGWAVE_NATIVE] Android Oboe listening: active {details}");
        let _ = startup_tx.send(Ok(details));

        while LISTENING.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }

        if let Err(error) = stream.stop() {
            eprintln!("[GGWAVE_NATIVE][WARN] Oboe stop input: {error:?}");
        }
        drop(stream);
        let _ = decoder.join();
        eprintln!("[GGWAVE_NATIVE] Android Oboe listening: stopped");
    });

    match startup_rx.recv_timeout(Duration::from_secs(4)) {
        Ok(Ok(details)) => {
            eprintln!("[GGWAVE_NATIVE] Android Oboe listen: confirmed {details}");
            Ok(())
        }
        Ok(Err(message)) => Err(anyhow!(message)),
        Err(error) => {
            LISTENING.store(false, Ordering::SeqCst);
            Err(anyhow!("Android Oboe listener startup timeout: {error}"))
        }
    }
}
