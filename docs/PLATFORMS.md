# Platform support

## Support policy

| Platform | ggwave-core | ggwave_dart | ggwave_flutter | Audio backend | Status |
|---|---|---|---|---|---|
| Android | Yes | Yes | Yes | native/CPAL via Rust | Tier 1 |
| iOS | Yes | Yes | Yes | native/CPAL via Rust | Tier 1 |
| macOS | Yes | Yes | Yes | native/CPAL via Rust | Tier 1 |
| Windows | Yes | Yes | Yes | native/CPAL via Rust | Tier 1 |
| Linux | Yes | Yes | Yes | native/CPAL via Rust | Tier 1 |
| Web | future WASM validation | Yes | Planned | Web Audio + WASM/JS | Tier 2 / planned |
| Fuchsia | not validated | Yes | Not planned yet | TBD | Unsupported |

## Why these five native platforms

They cover Flutter's mainstream mobile and desktop deployment targets and all have practical microphone/speaker paths. Keeping audio I/O outside `ggwave-core` prevents the codec crate from being coupled to CPAL or Flutter.

## Web

Web should not emulate the native implementation. A browser adapter should use `getUserMedia`, AudioWorklet/Web Audio for sample streaming, and either a WASM-compatible ggwave core or a small JS bridge. This keeps browser permission, latency and sample-rate behavior explicit.

## Validation gate

A platform is only promoted to supported after encode/decode smoke tests, microphone capture, speaker playback, audible roundtrip, ultrasonic roundtrip on representative hardware, permissions, lifecycle/resume, and release-mode build all pass.
