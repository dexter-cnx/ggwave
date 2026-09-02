# ggwave

A publish-oriented monorepo for mobile-tuned ggwave bindings across Rust, Dart, and Flutter.

## Packages

- `crates/ggwave-mobile` — Rust helpers for crates.io with mobile/ultrasonic tuning, payload limits, pre-emphasis, and duplicate suppression.
- `packages/ggwave_dart` — pure-Dart public contracts, protocol/tuning types, and sequence deduplication.
- `packages/ggwave_flutter` — Flutter transport backed by Rust and `flutter_rust_bridge` for microphone/speaker I/O.

The Bingo QR application used to exercise this stack lives separately and is intentionally not part of this package repository.

## Release order

1. Publish `ggwave-mobile` to crates.io.
2. Publish `ggwave_dart` to pub.dev.
3. Generate and validate FRB glue, then publish `ggwave_flutter` to pub.dev.

See [`RELEASE.md`](RELEASE.md) and run `./tool/release_check.sh` before publishing.

## Repository

https://github.com/dexter-cnx/ggwave

## License

MIT
