# ggwave

Universal, publish-oriented ggwave bindings for Rust, Dart, and Flutter.

## Packages

- `crates/ggwave-core` — platform-neutral Rust codec, protocols, tuning, ultrasonic helpers, and packet deduplication. It owns no microphone/speaker or Flutter APIs.
- `packages/ggwave_dart` — pure-Dart contracts, protocol/tuning types, and sequence deduplication.
- `packages/ggwave_flutter` — source directory for the publishable `ggwave_rs_flutter` package, backed by Rust/FRB and native audio I/O.

Application protocols such as Bingo, QR payloads, pairing formats, device IDs, or game state do **not** belong in this repository.

## Native platform targets

Tier 1 targets are Android, iOS, macOS, Windows, and Linux. Web is planned as a separate Web Audio + WASM/JS backend instead of reusing the native CPAL path. See [`docs/PLATFORMS.md`](docs/PLATFORMS.md).

Before the first Flutter publish, generate and commit the FRB/Cargokit platform scaffold:

```bash
./tool/bootstrap_flutter_native.sh
```

Then run the complete local release gate:

```bash
./tool/release_check.sh
```

## Release order

1. `ggwave-core` → crates.io
2. `ggwave_dart` → pub.dev
3. `ggwave_rs_flutter` → pub.dev

`ggwave_flutter` is already an existing third-party pub.dev package, so this repository intentionally uses `ggwave_rs_flutter` for its Flutter release name.

See [`RELEASE.md`](RELEASE.md) for the registry checklist and [`docs/ANDROID_VALIDATION.md`](docs/ANDROID_VALIDATION.md) for the first Tier 1 hardware gate.

## Repository

https://github.com/dexter-cnx/ggwave

## License

MIT
