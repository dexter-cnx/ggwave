//! Platform-neutral helpers built on top of `ggwave-rs`.
//!
//! This crate contains codec, protocol, tuning and optional packet-dedup logic.
//! It deliberately does not own microphones, speakers, Flutter, QR, or app
//! protocols. Audio I/O belongs in platform adapters such as `ggwave_flutter`.

#[cfg(feature = "dedup")]
use std::collections::VecDeque;
#[cfg(feature = "dedup")]
use std::time::{Duration, Instant};

use ggwave_rs::{
    default_parameters, ffi, set_rx_protocol_enabled, GgWave, ProtocolId, SampleFormat,
};
use thiserror::Error;

/// Maximum payload accepted by the wrapper.
pub const MAX_PAYLOAD: usize = 140;
/// Default ultrasonic start frequency in Hz.
pub const DEFAULT_ULTRASONIC_HZ: f32 = 12_000.0;
/// Legacy ultrasonic pre-emphasis setting retained for API compatibility.
///
/// The wrapper intentionally leaves ggwave's modulated waveform untouched.
pub const DEFAULT_PRE_EMPHASIS: f32 = 1.0;
/// Internal operating rate used by ggwave.
pub const DEFAULT_OPERATING_SAMPLE_RATE: f32 = 48_000.0;
/// FFT frame size used by upstream ggwave's default parameters.
pub const DEFAULT_SAMPLES_PER_FRAME: f32 = 1024.0;
/// ULTRASOUND_FAST occupies 96 FFT bins (16 bins × 2 bits × 3 bytes/chunk).
const ULTRASOUND_FAST_BINS: i32 = 96;
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

const ALL_NATIVE_PROTOCOLS: [ProtocolId; 12] = [
    ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_NORMAL,
    ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FAST,
    ProtocolId::GGWAVE_PROTOCOL_AUDIBLE_FASTEST,
    ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_NORMAL,
    ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST,
    ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FASTEST,
    ProtocolId::GGWAVE_PROTOCOL_DT_NORMAL,
    ProtocolId::GGWAVE_PROTOCOL_DT_FAST,
    ProtocolId::GGWAVE_PROTOCOL_DT_FASTEST,
    ProtocolId::GGWAVE_PROTOCOL_MT_NORMAL,
    ProtocolId::GGWAVE_PROTOCOL_MT_FAST,
    ProtocolId::GGWAVE_PROTOCOL_MT_FASTEST,
];

fn filter_rx_protocol(protocol: Protocol) {
    let selected = protocol.native();
    for candidate in ALL_NATIVE_PROTOCOLS {
        set_rx_protocol_enabled(candidate, candidate == selected);
    }
}

/// Codec tuning independent of any audio-device implementation.
#[derive(Debug, Clone, Copy)]
pub struct Tuning {
    pub ultrasonic_hz: f32,
    /// Retained for source compatibility. The encoder does not post-process the
    /// waveform because doing so changes the modem signal produced by ggwave.
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

        // ggwave_rx/txProtocolSetFreqStart() takes an FFT-bin index, NOT Hz.
        // At 48 kHz / 1024 samples one bin is 46.875 Hz, so 12 kHz is bin 256.
        let bin_hz = DEFAULT_OPERATING_SAMPLE_RATE / DEFAULT_SAMPLES_PER_FRAME;
        let bin = (self.ultrasonic_hz / bin_hz).round() as i32;
        let nyquist_bin = (DEFAULT_SAMPLES_PER_FRAME as i32) / 2;
        if bin < 1 || bin + ULTRASOUND_FAST_BINS > nyquist_bin {
            return Err(GgWaveError::InvalidFrequency);
        }

        unsafe {
            ffi::ggwave_rxProtocolSetFreqStart(ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST, bin);
            ffi::ggwave_txProtocolSetFreqStart(ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST, bin);
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
    pub fn with_sample_rate(tuning: Tuning, sample_rate: f32) -> Result<Self, GgWaveError> {
        Self::build(tuning, sample_rate, None)
    }

    /// Creates an RX codec that listens only for [protocol].
    ///
    /// Upstream protocol enablement is process-global and is consumed when a new
    /// ggwave instance is created. Filtering before construction prevents other
    /// protocol families from misattributing the same spectral window.
    pub fn with_sample_rate_and_rx_protocol(
        tuning: Tuning,
        sample_rate: f32,
        protocol: Protocol,
    ) -> Result<Self, GgWaveError> {
        Self::build(tuning, sample_rate, Some(protocol))
    }

    fn build(
        tuning: Tuning,
        sample_rate: f32,
        rx_protocol: Option<Protocol>,
    ) -> Result<Self, GgWaveError> {
        if sample_rate <= 0.0 {
            return Err(GgWaveError::InvalidSampleRate);
        }
        tuning.apply()?;
        if let Some(protocol) = rx_protocol {
            filter_rx_protocol(protocol);
        }
        let mut p = default_parameters();
        p.sampleRateInp = sample_rate;
        p.sampleRateOut = sample_rate;
        p.sampleRate = DEFAULT_OPERATING_SAMPLE_RATE;
        p.samplesPerFrame = DEFAULT_SAMPLES_PER_FRAME as i32;
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
        Ok(bytes_to_f32(&raw))
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

    fn assert_roundtrip(protocol: Protocol, tuning: Tuning) {
        let payload = b"hello";
        let tx = Codec::new(tuning).unwrap();
        let waveform = tx.encode(payload, protocol, 60).unwrap();
        let rx = Codec::with_sample_rate_and_rx_protocol(
            tuning,
            DEFAULT_OPERATING_SAMPLE_RATE,
            protocol,
        )
        .unwrap();

        let mut decoded = None;
        for chunk in waveform.chunks(DEFAULT_SAMPLES_PER_FRAME as usize) {
            if let Some(value) = rx.decode(chunk).unwrap() {
                decoded = Some(value);
                break;
            }
        }
        assert_eq!(decoded.as_deref(), Some(payload.as_slice()));
    }

    #[test]
    fn audible_encode_smoke() {
        let codec = Codec::new(Tuning::default()).unwrap();
        let wave = codec.encode(b"hello", Protocol::AudibleFast, 60).unwrap();
        assert!(!wave.is_empty());
    }

    #[test]
    fn ultrasonic_encode_smoke() {
        let codec = Codec::new(Tuning::default()).unwrap();
        let wave = codec
            .encode(b"hello", Protocol::UltrasonicFast, 60)
            .unwrap();
        assert!(!wave.is_empty());
    }

    #[test]
    fn audible_filtered_roundtrip() {
        assert_roundtrip(Protocol::AudibleFast, Tuning::default());
    }

    #[test]
    fn ultrasonic_12khz_filtered_roundtrip() {
        assert_roundtrip(Protocol::UltrasonicFast, Tuning::default());
    }

    #[test]
    fn validated_ultrasonic_profiles_map_to_expected_bins() {
        let bin_hz = DEFAULT_OPERATING_SAMPLE_RATE / DEFAULT_SAMPLES_PER_FRAME;
        for (hz, expected) in [(12_000.0, 256), (15_000.0, 320), (18_000.0, 384)] {
            assert_eq!((hz / bin_hz).round() as i32, expected);
        }
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
