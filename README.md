# ggwave

Universal, publish-oriented ggwave bindings for Rust, Dart, Flutter, and Kotlin/Android.

## Packages

- `crates/ggwave-core` — platform-neutral Rust codec, protocols, tuning, ultrasonic helpers, and packet deduplication. It owns no microphone/speaker or Flutter APIs.
- `crates/ggwave-jni` — JNI bridge that serializes native codec work onto a dedicated Rust thread for safe Kotlin/Java calls.
- `packages/ggwave_dart` — pure-Dart contracts, protocol/tuning types, and sequence deduplication.
- `packages/ggwave_flutter` — source directory for the publishable `ggwave_rs_flutter` package, backed by Rust/FRB and native audio I/O.
- `packages/ggwave_kotlin` — native Android/Kotlin AAR facade over the same Rust core, prepared for Maven publication.

Application protocols such as Bingo, QR payloads, pairing formats, device IDs, or game state do **not** belong in this repository.

## Consumer graph

```text
ggwave-core (Rust)
├── ggwave_rs_flutter -> Flutter/Dart apps
└── ggwave-jni -> ggwave-kotlin -> native Android/Kotlin apps
```

Both bindings use the same protocol IDs, payload limits, ultrasonic tuning, and codec implementation.

## Flutter 3.47 / Android

`ggwave_rs_flutter` now requires Flutter 3.47+ / Dart 3.12+. The Android/Kotlin library is structured for AGP 9 built-in Kotlin (`android.builtInKotlin=true`) and does not apply the legacy `org.jetbrains.kotlin.android` plugin.

## Native platform targets

Tier 1 Flutter targets are Android, iOS, macOS, Windows, and Linux. Web is planned as a separate Web Audio + WASM/JS backend instead of reusing the native CPAL path. See [`docs/PLATFORMS.md`](docs/PLATFORMS.md).

The Kotlin binding targets Android with `arm64-v8a`, `armeabi-v7a`, and `x86_64` Rust libraries.

Before the first Flutter publish, generate and commit the FRB/Cargokit platform scaffold:

```bash
./tool/bootstrap_flutter_native.sh
```

Then run the Flutter/Rust/Dart release gate:

```bash
./tool/release_check.sh
```

For the Kotlin AAR/Maven artifact run:

```bash
./tool/release_kotlin_check.sh
```

## Release order

1. `ggwave-core` → crates.io
2. `ggwave_dart` → pub.dev
3. `ggwave_rs_flutter` → pub.dev
4. `ggwave-kotlin` → Maven repository

`ggwave_flutter` is already an existing third-party pub.dev package, so this repository intentionally uses `ggwave_rs_flutter` for its Flutter release name.

See [`RELEASE.md`](RELEASE.md), [`docs/ANDROID_VALIDATION.md`](docs/ANDROID_VALIDATION.md), and [`packages/ggwave_kotlin/README.md`](packages/ggwave_kotlin/README.md).

## Repository

https://github.com/dexter-cnx/ggwave

## License

MIT
