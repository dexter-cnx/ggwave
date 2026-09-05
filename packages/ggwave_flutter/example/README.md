# ggwave_rs_flutter Android validation example

This example is the physical Android acoustic-validation UI for `ggwave_rs_flutter`.

## VSCode workflow

From the repository root:

1. Open the repository in VSCode.
2. Connect a physical Android device with USB debugging enabled.
3. Select the device in the Flutter device picker.
4. Open **Run and Debug**.
5. Select **ggwave Android Validation**.
6. Press **F5**.

The launch configuration runs `tool/prepare_flutter_android_validation.sh` first. The task:

- installs `flutter_rust_bridge_codegen` 2.8.0 through Cargo if it is missing;
- generates the FRB/Cargokit native plugin scaffold;
- normalizes the generated Android plugin to compileSdk 36;
- generates the example Android runner when needed;
- adds `android.permission.RECORD_AUDIO` to the generated example manifest;
- resolves Flutter dependencies.

VSCode then runs `example/lib/main.dart` directly on the selected device. There is no need to download or copy a CI APK.

## Validation UI

The app supports:

- **TX / Send** and **RX / Listen** roles;
- Audible Fast;
- Ultrasonic Fast at 12 kHz, 15 kHz, and 18 kHz;
- editable payloads up to 140 bytes;
- runtime microphone permission;
- Start/Stop Listening;
- sent/received counters;
- last received payload and status.

Use two physical Android devices. Select the same protocol/frequency on both devices, set one to TX and the other to RX, test the transmission, then reverse roles.
