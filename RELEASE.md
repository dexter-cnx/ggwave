# Release guide

## Order

1. `ggwave-core` on crates.io
2. `ggwave_dart` on pub.dev
3. `ggwave_flutter` on pub.dev

Run `./tool/release_check.sh` before any registry publish.

## Rust

```bash
cd crates/ggwave-core
cargo fmt --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo publish --dry-run
```

The core crate deliberately has no CPAL or Flutter dependency. `ultrasonic` and `dedup` are default features and can be disabled for smaller integrations.

## Dart

```bash
cd packages/ggwave_dart
dart pub get
dart analyze
dart test
dart pub publish --dry-run
```

## Flutter

Generate FRB glue first, then validate Android, iOS, macOS, Windows and Linux. The release guard refuses to publish while generated Dart remains a placeholder.

Web is not claimed as supported in 1.2.0; it requires a Web Audio/WASM adapter.
