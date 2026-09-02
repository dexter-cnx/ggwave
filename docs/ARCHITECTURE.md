# Architecture

## Boundary

`ggwave-core` is the codec boundary. It has no Flutter, CPAL, QR or application dependency.

`ggwave_flutter_native` is an adapter. It owns microphone capture, speaker playback, lifecycle and Flutter Rust Bridge streaming. It calls `ggwave-core` for encode/decode/tuning/dedup instead of implementing those rules itself.

`ggwave_dart` defines portable Dart-facing contracts. Applications define only payload semantics.

## Local Rust development

The repository root is a Cargo workspace. `[patch.crates-io]` maps `ggwave-core` to the local crate so `ggwave_flutter_native` can be checked before `ggwave-core` is published. Registry consumers still resolve version `1.2.0` after release.
