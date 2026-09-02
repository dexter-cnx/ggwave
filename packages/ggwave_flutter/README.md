# ggwave_flutter

Flutter transport for ggwave using Rust + `flutter_rust_bridge` 2.8.0.

Features: encode/decode transport, microphone receive, speaker playback, 12–19 kHz tuning, ultrasonic volume auto-boost, 1.8x pre-emphasis, and 800 ms receive deduplication.

## Release prerequisite

Publish `ggwave-mobile 1.2.0` to crates.io and `ggwave_dart 1.2.0` to pub.dev first. Then regenerate FRB glue on a machine with Flutter + Rust:

```bash
flutter pub get
cargo install flutter_rust_bridge_codegen --version 2.8.0
flutter_rust_bridge_codegen generate
flutter analyze
flutter test
dart pub publish --dry-run
```

The checked-in `lib/src/rust/api.dart` is a deliberately loud placeholder so the source kit remains analyzable before codegen; **do not publish that placeholder**. `tool/release_check.sh` fails if it is still present.
