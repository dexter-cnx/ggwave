# VSCode Android validation walkthrough

The Flutter example can be deployed directly from VSCode to a connected Android device. This avoids downloading and manually copying CI APK artifacts.

## Launch path

The VSCode configuration is `.vscode/launch.json`:

```text
ggwave Android Validation
```

Before Dart/Flutter launches the app, `.vscode/tasks.json` runs:

```text
Prepare ggwave Android validation
```

which invokes:

```text
tool/prepare_flutter_android_validation.sh
```

## Preparation sequence

The preparation script intentionally reuses the same FRB bootstrap contract as CI:

```text
check Flutter / Cargo / Python 3
      ↓
install flutter_rust_bridge_codegen 2.8.0 when missing
      ↓
tool/bootstrap_flutter_native.sh
      ↓
FRB integrate plugin template
      ↓
preserve package pubspec/public barrel
remove FRB simple/integration template artifacts
normalize generated Android plugin compileSdk to 36
FRB generate from crate::api
      ↓
create example/android when missing
      ↓
insert RECORD_AUDIO in generated example manifest
      ↓
flutter pub get
      ↓
VSCode runs example/lib/main.dart on selected device
```

Generated native folders are development artifacts and are ignored by `.gitignore`.

## Validation UI data flow

The example does not bypass the public package API. It creates a `GgWaveFlutterTransport` and uses only its public operations.

TX:

```text
payload text
 -> UTF-8 Uint8List
 -> GgWaveFlutterTransport.encode()
 -> FRB
 -> Rust ggwave codec
 -> Float32 waveform
 -> GgWaveFlutterTransport.play()
 -> speaker
```

RX:

```text
Permission.microphone.request()
 -> GgWaveFlutterTransport.startListening()
 -> native microphone / Rust decoder
 -> transport messages stream
 -> received counter + last payload UI
```

Ultrasonic profiles call `setUltrasonicFrequency()` before TX or RX so both devices can explicitly select 12 kHz, 15 kHz, or 18 kHz.

## Physical test

Run the same example on two physical Android devices, set one to TX and one to RX, choose the same protocol/frequency, test a payload, then reverse roles. Detailed acceptance criteria are in `docs/ANDROID_VALIDATION.md`.
