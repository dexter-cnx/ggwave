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

This is **build validation only**. Physical-device testing is still required before claiming acoustic hardware validation.

### Flutter Android — build validated, hardware validation pending

GitHub Actions workflow `Flutter Android #32`, run `33622377147`, passed on `main` after PR #1 merged. The complete Flutter Android build gate validated Flutter 3.47.0, FRB 2.8.0 generation, Dart/Flutter analysis and tests, Android example build, and artifact upload.

The Flutter example now also serves as a physical hardware-validation UI. Hardware validation remains pending until representative two-device acoustic results are recorded.

## Run the Flutter validation app from VSCode

For local device testing, prefer the repository VSCode launch configuration instead of downloading and copying a CI APK.

1. Open the repository root in VSCode.
2. Connect a physical Android device with USB debugging enabled.
3. Select the device from the Flutter device picker.
4. Open **Run and Debug**.
5. Select **ggwave Android Validation**.
6. Press **F5**.

The launch configuration runs `tool/prepare_flutter_android_validation.sh` as a `preLaunchTask`. It:

- installs `flutter_rust_bridge_codegen` 2.8.0 through Cargo when missing;
- generates the FRB/Cargokit native plugin scaffold;
- normalizes the generated Android plugin to compileSdk 36;
- removes FRB template-only demo/integration artifacts and restores the package-owned manifest/barrel;
- generates the example Android runner when needed;
- adds `android.permission.RECORD_AUDIO` to the generated example manifest;
- resolves Flutter dependencies.

VSCode then runs `packages/ggwave_flutter/example/lib/main.dart` directly on the selected physical device. No CI artifact download or manual APK copy is required.

The validation UI provides:

- TX / Send mode;
- RX / Listen mode;
- Audible Fast;
- Ultrasonic 12 kHz;
- Ultrasonic 15 kHz;
- Ultrasonic 18 kHz;
- payload entry up to 140 bytes;
- runtime microphone permission;
- Start/Stop Listening;
- sent/received counters;
- last received payload and current status.

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
- Rust/Cargo available on PATH
- Android SDK 36
- Android NDK 27
- Python 3 for the local validation bootstrap manifest patch
- physical Android device with microphone and speaker for hardware acceptance

The VSCode preLaunch task installs FRB codegen automatically if it is not already present.

## Physical-device procedure

Use two Android devices, A and B.

1. Run the validation app on both devices from VSCode.
2. On B, select **RX / Listen**, select the desired profile, then tap **Start Listening** and grant microphone permission.
3. On A, select **TX / Send**, choose the same profile, enter a payload, and tap **Send**.
4. Confirm B increments its receive counter and displays the exact payload.
5. Repeat with Audible Fast and Ultrasonic 12 kHz, 15 kHz, and 18 kHz.
6. Reverse roles: A listens and B transmits.
7. Repeat at approximately 0.5 m, 1 m and 2 m where practical.
8. Test speaker-to-mic facing and normal handheld orientations.
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
