# Android Tier 1 validation

Android is the reference mobile target for `ggwave_rs_flutter`, and `ggwave-kotlin` provides the native Android/Kotlin path over the same Rust codec core.

## Current status

### Kotlin/Android AAR — build validated

GitHub Actions workflow `Kotlin Android #18`, run `33600474569`, passed on 2026-09-02. The complete build gate validated:

- Rust formatting;
- host JNI `cargo check` and `clippy -D warnings`;
- release cross-compilation for `arm64-v8a`, `armeabi-v7a`, and `x86_64`;
- AGP 9.0.0 built-in Kotlin with Gradle 9.1;
- release AAR assembly;
- Maven POM generation;
- standalone Android validation app compilation against the public Kotlin API;
- upload of both the release AAR bundle and standalone validation APK.

Artifacts from run `33600474569`:

```text
ggwave-kotlin-release
SHA-256: 0b34957c62ac752d1856584bf543007b53d59e981bfc4f1e3c5c2721d2c3606f
```

```text
ggwave-kotlin-validation-apk
SHA-256: 6ca205abe5c17150be07e1b59d1c6f23b41b96b9adb45f58728c7750cb737be1
```

The standalone APK consumes the public Kotlin API and is intended for the two-device procedure below, so physical-device testing does not require creating a separate host application.

This is **build validation only**. The physical-device acceptance section below is still required before claiming acoustic hardware validation.

### Flutter Android — source implemented, build/hardware validation pending

The Flutter package shares `ggwave-core`, but its FRB/Cargokit/native-platform build gate and physical Android acceptance are tracked separately from the Kotlin AAR gate.

The dedicated `Flutter Android` workflow is pinned to Flutter 3.47.0, FRB 2.8.0, Android SDK 36, and NDK 27. Its build gate generates the FRB/Cargokit scaffold, formats/analyzes/tests both Dart packages, builds the Android example APK, and uploads that APK as a workflow artifact. Until the complete workflow passes, Flutter Android remains source implemented but build validation pending.

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

Build the standalone validation app locally with:

```bash
gradle -p examples/ggwave_kotlin_validation :app:assembleDebug
```

Or download the `ggwave-kotlin-validation-apk` artifact from successful run `33600474569`.

### Flutter Android

- Flutter >= 3.47
- Dart >= 3.12
- Rust >= 1.77
- Android SDK 36
- Android NDK 27
- `flutter_rust_bridge_codegen` 2.8.0 installed as a Cargo binary
- Linux host builds additionally need ALSA development headers and `pkg-config` because FRB `cargo expand` compiles the Rust crate on the host before Android cross-compilation
- physical Android device with microphone and speaker for hardware acceptance

Generate the native plugin scaffold with:

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

The CI workflow also builds the example APK so source implementation, build validation, and hardware validation remain independently evidenced.

## Physical-device procedure

Use two Android devices, A and B.

1. Install the validation APK on both devices.
2. Grant microphone permission on both devices.
3. On B, tap Start Listening.
4. On A, send an Audible packet and confirm B increments its receive counter and displays the payload.
5. Repeat with Ultrasonic 12 kHz, 15 kHz and 18 kHz.
6. Reverse roles: A listens and B transmits.
7. Repeat at approximately 0.5 m, 1 m and 2 m where practical.
8. Test at least speaker-to-mic facing and normal handheld orientations.
9. Background and resume the receiving app, then confirm listening can be restarted cleanly.
10. Execute ten start/listen/stop cycles without crash, stuck audio state or leaked playback.

Record for each run:

```text
TX device / Android version
RX device / Android version
protocol
frequency
volume
distance
orientation
attempts / successes
audible artifact notes
lifecycle notes
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
