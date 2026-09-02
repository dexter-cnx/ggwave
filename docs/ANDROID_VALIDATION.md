# Android Tier 1 validation

Android is the reference mobile target for `ggwave_rs_flutter`, and `ggwave-kotlin` provides the native Android/Kotlin path over the same Rust codec core.

## Current status

### Kotlin/Android AAR — build validated

GitHub Actions workflow run `33598931434` passed on 2026-09-02. The release gate validated:

- Rust formatting;
- host JNI `cargo check` and `clippy -D warnings`;
- release cross-compilation for `arm64-v8a`, `armeabi-v7a`, and `x86_64`;
- AGP 9.0.0 built-in Kotlin with Gradle 9.1;
- release AAR assembly;
- Maven POM generation;
- upload of the `ggwave-kotlin-release` artifact.

Artifact SHA-256 from that run:

```text
4b5a4bdf23c38ec10ebd44897e69226e12da099fbacf7f658877b189573e02db
```

This is **build validation only**. The physical-device acceptance section below is still required before claiming acoustic hardware validation.

### Flutter Android — implementation present, full release/hardware validation pending

The Flutter package shares `ggwave-core`, but its FRB/Cargokit/native-platform release gate and physical Android acceptance are tracked separately from the Kotlin AAR gate.

## Build prerequisites

### Kotlin/Android

- Rust >= 1.77
- Android SDK 36
- Android NDK 27
- AGP 9.0.0
- Gradle 9.1+
- `cargo-ndk`

Run:

```bash
bash ./tool/release_kotlin_check.sh
```

### Flutter Android

- Flutter >= 3.47
- Dart >= 3.12
- Rust >= 1.77
- Android SDK + NDK
- `flutter_rust_bridge_codegen` 2.8.0
- physical Android device with microphone and speaker

Generate the native plugin scaffold once with:

```bash
./tool/bootstrap_flutter_native.sh
```

Then validate:

```bash
cargo fmt --check --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd packages/ggwave_flutter
flutter pub get
flutter analyze
flutter test
cd example
flutter run
```

## Hardware acceptance

A release may claim Android acoustic support only after all of these pass on representative physical devices:

1. microphone permission request succeeds;
2. input stream starts/stops repeatedly without crash;
3. audible encode/play/decode roundtrip succeeds;
4. ultrasonic roundtrip succeeds at 12 kHz;
5. 15 kHz and 18 kHz are measured separately because speaker/microphone response varies strongly by device;
6. app background/resume does not leave the audio stream stuck;
7. ten consecutive start/listen/stop cycles pass;
8. release APK/AAB builds successfully for Flutter consumers, and a release AAR consumer app installs/runs successfully for Kotlin consumers.

For meaningful acoustic evidence, test with two physical devices where one transmits and the other receives, then reverse direction. Record device models, Android versions, selected frequency, distance, orientation, pass/fail count, and any audible artifacts.

## Tuning guidance

- 12 kHz: strongest compatibility and range, but faintly audible to some users.
- 15 kHz: quieter compromise; younger users may still hear it.
- 18 kHz: least audible but short-range and hardware-sensitive.
- ultrasonic transmissions default to higher volume than audible transmissions.

Treat these as starting profiles, not guaranteed distance specifications. Acoustic range depends on device hardware, orientation, room reflections, noise, case/cover, and OS audio processing.
