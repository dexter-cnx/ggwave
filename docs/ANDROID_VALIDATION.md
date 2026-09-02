# Android Tier 1 validation

Android is the reference mobile target for `ggwave_flutter`.

## Build prerequisites

- Flutter >= 3.22
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

A release may claim Android support only after all of these pass on a physical device:

1. microphone permission request succeeds;
2. input stream starts/stops repeatedly without crash;
3. audible encode/play/decode roundtrip succeeds;
4. ultrasonic roundtrip succeeds at 12 kHz;
5. 15 kHz and 18 kHz are measured separately because speaker/microphone response varies strongly by device;
6. app background/resume does not leave the audio stream stuck;
7. ten consecutive start/listen/stop cycles pass;
8. release APK/AAB builds successfully.

## Tuning guidance

- 12 kHz: strongest compatibility and range, but faintly audible to some users.
- 15 kHz: quieter compromise; younger users may still hear it.
- 18 kHz: least audible but short-range and hardware-sensitive.
- ultrasonic transmissions default to higher volume than audible transmissions.

Treat these as starting profiles, not guaranteed distance specifications. Acoustic range depends on device hardware, orientation, room reflections, noise, case/cover, and OS audio processing.
