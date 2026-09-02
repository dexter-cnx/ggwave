# ggwave-mobile

Mobile-tuned Rust helpers for [ggwave](https://github.com/ggerganov/ggwave), built on `ggwave-rs`.

## What it adds

- App protocol IDs `1 = AudibleFast`, `5 = UltrasonicFast`.
- Tunable ultrasonic start frequency, 8–19 kHz; default 12 kHz.
- 1.8x high-frequency pre-emphasis with `[-1, 1]` clamping.
- 140-byte mobile payload guard.
- 800 ms sliding-window packet deduplicator.
- Explicitly preserves upstream single-thread ownership requirements.

Typical tuning: 12 kHz is often usable over a few metres but faint; 15 kHz is quieter to adults but can be audible to children; 18 kHz is much less audible but phone-to-phone range may fall to roughly tens of centimetres. Treat these as starting points, not guarantees: hardware and room acoustics dominate.

```rust
use ggwave_mobile::{MobileGgWave, MobileProtocol, MobileTuning};

let codec = MobileGgWave::new(MobileTuning::default())?;
let waveform = codec.encode(b"hello", MobileProtocol::UltrasonicFast, 85)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Release validation

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo package
cargo publish --dry-run
```

MIT licensed. Upstream `ggwave-rs` is a separate MIT-licensed dependency.
