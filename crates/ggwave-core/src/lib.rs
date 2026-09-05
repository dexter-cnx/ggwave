//! Platform-neutral helpers built on top of `ggwave-rs`.
//!
//! This crate contains codec, protocol, tuning and optional packet-dedup logic.
//! It deliberately does not own microphones, speakers, Flutter, QR, or app
//! protocols. Audio I/O belongs in platform adapters such as `ggwave_flutter`.

#[cfg(feature = "dedup")]
use std::collections::VecDeque;
#[cfg(feature = "dedup")]
use std::time::{Duration, Instant};

use ggwave_rs::{default_parameters, ffi, GgWave, ProtocolId, SampleFormat};
use thiserror::Error;

/// Maximum payload accepted by the wrapper.
pub const MAX_PAYLOAD: usize = 140;
/// Default ultrasonic start frequency in Hz.
pub const DEFAULT_ULTRASONIC_HZ: f32 = 12_000.0;
/// Default ultrasonic pre-emphasis gain.
pub const DEFAULT_PRE_EMPHASIS: f32 = 1.8;
/// Internal operating rate used by ggwave. Device input/output rates may differ;
/// upstream ggwave resamples between those rates and this operating rate.
pub const DEFAULT_OPERATING_SAMPLE_RATE: f32 = 48_000.0;
#[cfg(feature = "dedup")]
/// Default duplicate-suppression window.
pub const DEFAULT_DEDUP_WINDOW: Duration = Duration::from_millis(800);

#[derive(Debug, Error)]
pub enum GgWaveError {
    #[error("payload exceeds {MAX_PAYLOAD} bytes")]
    PayloadTooLarge,
    #[error("volume must be in 0..=100")]
    InvalidVolume,
    #[error("ultrasonic frequency must be in 8000..=19000 Hz")]
    InvalidFrequency,
    #[error("sample rate must be greater than zero")]
    InvalidSampleRate,
    #[error("ggwave error: {0}")]
    Codec(String),
}

/// Stable public protocol IDs shared with Dart/Flutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    AudibleFast = 1,
    UltrasonicFast = 5,
}

impl Protocol {
    fn native(self) -> ProtocolId {
        match self {
            Self::AudibleFast => ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
            Self::UltrasonicFast => ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST,
        }
    }

    pub const fn is_ultrasonic(self) -> bool {
        matches!(self, Self::UltrasonicFast)
    }

    pub const fn from_app_id(id: i32) -> Self {
        match id {
            5 => Self::UltrasonicFast,
            _ => Self::AudibleFast,
        }
    }
}

/// Codec tuning independent of any audio-device implementation.
#[derive(Debug, Clone, Copy)]
pub struct Tuning {
    pub ultrasonic_hz: f32,
    pub ultrasonic_pre_emphasis: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            ultrasonic_hz: DEFAULT_ULTRASONIC_HZ,
            ultrasonic_pre_emphasis: DEFAULT_PRE_EMPHASIS,
        }
    }
}

impl Tuning {
    pub fn validate(self) -> Result<(), GgWaveError> {
        if !(8_000.0..=19_000.0).contains(&self.ultrasonic_hz) {
            return Err(GgWaveError::InvalidFrequency);
        }
        Ok(())
    }

    #[cfg(feature = "ultrasonic")]
    fn apply(self) -> Result<(), GgWaveError> {
        self.validate()?;
        let hz = self.ultrasonic_hz.round() as i32;
        unsafe {
            ffi::ggwave_rxProtocolSetFreqStart(ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST, hz);
            ffi::ggwave_txProtocolSetFreqStart(ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST, hz);
        }
        Ok(())
    }

    #[cfg(not(feature = "ultrasonic"))]
    fn apply(self) -> Result<(), GgWaveError> {
        self.validate()
    }
}

/// Single-threaded ggwave codec. Upstream `GgWave` is !Send/!Sync.
pub struct Codec {
    inner: GgWave,
    tuning: Tuning,
}

impl Codec {
    pub fn new(tuning: Tuning) -> Result<Self, GgWaveError> {
        Self::with_sample_rate(tuning, DEFAULT_OPERATING_SAMPLE_RATE)
    }

    /// Creates a codec for a device whose capture/playback rate is
    /// [sample_rate], while keeping ggwave's internal operating rate at 48 kHz.
    ///
    /// This distinction is important on Android where devices commonly expose
    /// 44.1 kHz input. ggwave performs the required resampling internally when
    /// `sampleRateInp`/`sampleRateOut` differ from `sampleRate`.
    pub fn with_sample_rate(tuning: Tuning, sample_rate: f32) -> Result<Self, GgWaveError> {
        if sample_rate <= 0.0 {
            return Err(GgWaveError::InvalidSampleRate);
        }
        tuning.apply()?;
        let mut p = default_parameters();
        p.sampleRateInp = sample_rate;
        p.sampleRateOut = sample_rate;
        p.sampleRate = DEFAULT_OPERATING_SAMPLE_RATE;
        p.sampleFormatInp = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
        p.sampleFormatOut = SampleFormat::GGWAVE_SAMPLE_FORMAT_F32;
        let inner = GgWave::new(p).map_err(|e| GgWaveError::Codec(e.to_string()))?;
        Ok(Self { inner, tuning })
    }

    pub fn encode(
        &self,
        payload: &[u8],
        protocol: Protocol,
        volume: i32,
    ) -> Result<Vec<f32>, GgWaveError> {
        if payload.len() > MAX_PAYLOAD {
            return Err(GgWaveError::PayloadTooLarge);
        }
        if !(0..=100).contains(&volume) {
            return Err(GgWaveError::InvalidVolume);
        }
        self.tuning.apply()?;
        let raw = self
            .inner
            .encode(payload, protocol.native(), volume)
            .map_err(|e| GgWaveError::Codec(e.to_string()))?;
        let mut samples = bytes_to_f32(&raw);
        #[cfg(feature = "ultrasonic")]
        if protocol.is_ultrasonic() {
            pre_emphasis(&mut samples, self.tuning.ultrasonic_pre_emphasis);
        }
        Ok(samples)
    }

    pub fn decode(&self, samples: &[f32]) -> Result<Option<Vec<u8>>, GgWaveError> {
        let raw = f32_to_bytes(samples);
        self.inner
            .decode(&raw)
            .map_err(|e| GgWaveError::Codec(e.to_string()))
    }
}

#[cfg(feature = "dedup")]
pub struct PacketDeduper {
    window: Duration,
    seen: VecDeque<(Vec<u8>, Instant)>,
}

#[cfg(feature = "dedup")]
impl Default for PacketDeduper {
    fn default() -> Self {
        Self::new(DEFAULT_DEDUP_WINDOW)
    }
}

#[cfg(feature = "dedup")]
impl PacketDeduper {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            seen: VecDeque::new(),
        }
    }

    pub fn accept(&mut self, payload: &[u8]) -> bool {
        let now = Instant::now();
        while self
            .seen
            .front()
            .is_some_and(|(_, at)| now.duration_since(*at) > self.window)
        {
            self.seen.pop_front();
        }
        if self.seen.iter().any(|(seen, _)| seen.as_slice() == payload) {
            return false;
        }
        self.seen.push_back((payload.to_vec(), now));
        true
    }
}

#[cfg(feature = "ultrasonic")]
fn pre_emphasis(samples: &mut [f32], gain: f32) {
    let mut previous = 0.0f32;
    for sample in samples {
        let input = *sample;
        *sample = ((input - 0.85 * previous) * gain).clamp(-1.0, 1.0);
        previous = input;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audible_encode_smoke() {
        let codec = Codec::new(Tuning::default()).unwrap();
        let wave = codec.encode(b"hello", Protocol::AudibleFast, 60).unwrap();
        assert!(!wave.is_empty());
    }

    #[test]
    fn protocol_ids_are_stable() {
        assert_eq!(Protocol::AudibleFast as i32, 1);
        assert_eq!(Protocol::UltrasonicFast as i32, 5);
    }

    #[test]
    fn accepts_common_android_input_rate() {
        let codec = Codec::with_sample_rate(Tuning::default(), 44_100.0);
        assert!(codec.is_ok());
    }
}
