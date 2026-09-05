# ggwave

Universal, publish-oriented ggwave bindings for Rust, Dart, Flutter, and Kotlin/Android.

## Packages

- `crates/ggwave-core` — platform-neutral Rust codec, protocols, tuning, ultrasonic helpers, and packet deduplication. It owns no microphone/speaker or Flutter APIs.
- `crates/ggwave-jni` — JNI bridge that serializes native codec work onto a dedicated Rust thread for safe Kotlin/Java calls.
- `packages/ggwave_dart` — pure-Dart contracts, protocol/tuning types, and sequence deduplication.
- `packages/ggwave_flutter` — source directory for the publishable `ggwave_rs_flutter` package, backed by Rust/FRB and native audio I/O.
- `packages/ggwave_kotlin` — native Android/Kotlin AAR facade over the same Rust core, prepared for Maven publication.
- `examples/ggwave_kotlin_validation` — standalone native Android hardware-validation app that consumes the public Kotlin API.

Application protocols such as Bingo, QR payloads, pairing formats, device IDs, or game state do **not** belong in this repository.

## Consumer graph

```text
ggwave-core (Rust)
├── ggwave_rs_flutter -> Flutter/Dart apps
└── ggwave-jni -> ggwave-kotlin -> native Android/Kotlin apps
```

Both bindings use the same protocol IDs, payload limits, ultrasonic tuning, and codec implementation.

## Flutter 3.47 / Android

`ggwave_rs_flutter` requires Flutter 3.47+ / Dart 3.12+. The Android/Kotlin library is structured for AGP 9 built-in Kotlin (`android.builtInKotlin=true`) and does not apply the legacy `org.jetbrains.kotlin.android` plugin.

The Kotlin Android build uses compileSdk 36, minSdk 23, AGP 9.0.0 and Gradle 9.1+.

Flutter Android CI is pinned to Flutter 3.47.0 and FRB 2.8.0 so compatibility is tested against the declared baseline rather than a floating stable SDK. FRB 2.8's plugin template is normalized to compileSdk 36 after integration so its generated Android scaffold remains compatible with the Flutter 3.47 AndroidX dependency set.

### Run the Flutter validation app from VSCode

For physical Android testing, you do not need to download or copy a CI APK.

1. Open the repository root in VSCode.
2. Connect an Android device with USB debugging enabled.
3. Select the device in the Flutter device picker.
4. Open **Run and Debug**.
5. Select **ggwave Android Validation**.
6. Press **F5**.

The pre-launch task prepares FRB/native scaffolding, the Android example runner, microphone permission, and dependencies automatically. The example UI provides TX/RX roles, Audible Fast, Ultrasonic 12/15/18 kHz, payload entry, Start/Stop Listening, sent/received counters, and the last received payload.

See `packages/ggwave_flutter/example/README.md` and `docs/ANDROID_VALIDATION.md` for the two-device acceptance procedure.

## Validation status

Kotlin/Android is **build validated** on GitHub Actions as of 2026-09-02:

- Rust formatting, host `cargo check`, and `clippy -D warnings` pass;
- JNI cross-compiles for `arm64-v8a`, `armeabi-v7a`, and `x86_64`;
- AGP 9 built-in Kotlin release AAR builds successfully with Gradle 9.1;
- Maven POM generation succeeds;
- standalone validation app compiles successfully against the public AAR-facing API;
- workflow `Kotlin Android #18`, run `33600474569`, passed the complete Android gate and produced both artifacts.

Artifacts from run `33600474569`:

- `ggwave-kotlin-release` — release AAR, SHA-256 `0b34957c62ac752d1856584bf543007b53d59e981bfc4f1e3c5c2721d2c3606f`;
- `ggwave-kotlin-validation-apk` — debug APK for physical two-device acoustic validation, SHA-256 `6ca205abe5c17150be07e1b59d1c6f23b41b96b9adb45f58728c7750cb737be1`.

Flutter/Android is also **build validated**. Post-merge workflow `Flutter Android #32`, run `33622377147`, passed on `main`, including FRB generation, Dart/Flutter analysis and tests, Android example build, and artifact upload.

Neither Android binding is yet **acoustic hardware validated**. Physical-device audible/ultrasonic roundtrip, permission, lifecycle, distance/orientation, and repeated start/stop acceptance remain tracked in `docs/ANDROID_VALIDATION.md`.

## Native platform targets

Tier 1 Flutter targets are Android, iOS, macOS, Windows, and Linux. Android is build validated; iOS, macOS, Windows, and Linux remain source implemented with build validation pending. Web is planned as a separate Web Audio + WASM/JS backend instead of reusing the native CPAL path. See [`docs/PLATFORMS.md`](docs/PLATFORMS.md).

The Kotlin binding targets Android with `arm64-v8a`, `armeabi-v7a`, and `x86_64` Rust libraries.

## Android validation app

The Flutter validation example is the preferred local path for Flutter hardware testing because VSCode can deploy it directly to connected devices. The native Kotlin validation APK remains available for Kotlin/JNI-specific validation.

See [`docs/ANDROID_VALIDATION.md`](docs/ANDROID_VALIDATION.md) for the acceptance matrix.

## Release checks

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

For architecture and code flow, see [`CODE_WALKTHROUGH.md`](CODE_WALKTHROUGH.md). Also see [`RELEASE.md`](RELEASE.md), [`docs/ANDROID_VALIDATION.md`](docs/ANDROID_VALIDATION.md), [`docs/PLATFORMS.md`](docs/PLATFORMS.md), and [`packages/ggwave_kotlin/README.md`](packages/ggwave_kotlin/README.md).

## Repository

https://github.com/dexter-cnx/ggwave

## License

MIT
