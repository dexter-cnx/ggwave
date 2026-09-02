# ggwave

Universal, reusable ggwave packages for Rust, Dart and Flutter. The repository is application-agnostic: it contains no Bingo, QR, game ID, player ID or app-specific packet semantics.

## Packages

- `crates/ggwave-core` — platform-neutral Rust codec, protocol/tuning primitives and optional packet deduplication.
- `packages/ggwave_dart` — pure-Dart public contracts, protocol/tuning types and sequence deduplication.
- `packages/ggwave_flutter` — Flutter microphone/speaker adapter backed by Rust + Flutter Rust Bridge.

## Architecture

`ggwave-core` owns codec behavior. Platform adapters own audio I/O. Applications only define their own payload bytes.

```text
application
    -> ggwave_flutter / another adapter
        -> ggwave_dart contracts
        -> ggwave-core codec
            -> ggwave-rs
```

## Platform targets

First-class target set: Android, iOS, macOS, Windows and Linux. Pure Dart APIs are usable anywhere Dart runs. Web is a separate planned backend using Web Audio + WASM/JS because browser microphone/speaker access is not the same native `cpal` path.

See [`docs/PLATFORMS.md`](docs/PLATFORMS.md).

## Release order

1. Publish `ggwave-core` to crates.io.
2. Publish `ggwave_dart` to pub.dev.
3. Generate/validate FRB glue and native builds, then publish `ggwave_flutter` to pub.dev.

See [`RELEASE.md`](RELEASE.md) and run `./tool/release_check.sh` before publishing.

## License

MIT
