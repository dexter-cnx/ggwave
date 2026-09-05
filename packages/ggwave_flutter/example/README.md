# ggwave_rs_flutter examples

This Flutter example project contains two physical-device entrypoints built on the public `ggwave_rs_flutter` API.

## 1. Android validation UI

Use this entrypoint for low-level acoustic validation:

```text
lib/main.dart
```

From the repository root in VSCode:

1. Connect a physical Android device with USB debugging enabled.
2. Select the device in the Flutter device picker.
3. Open **Run and Debug**.
4. Select **ggwave Android Validation**.
5. Press **F5**.

The launch configuration runs `tool/prepare_flutter_android_validation.sh` first. The task:

- installs `flutter_rust_bridge_codegen` 2.8.0 through Cargo if it is missing;
- generates the FRB/Cargokit native plugin scaffold;
- normalizes the generated Android plugin to compileSdk 36;
- generates the example Android runner when needed;
- adds `android.permission.RECORD_AUDIO` to the generated example manifest;
- resolves Flutter dependencies.

The validation UI supports TX/RX roles, Audible Fast, Ultrasonic Fast at 12/15/18 kHz, editable payloads, runtime microphone permission, listening controls, counters, and the last received payload.

## 2. Tic-Tac-Toe 1 ↔ 1 reference demo

Use this entrypoint to demonstrate interactive peer-to-peer application traffic over ggwave:

```text
lib/tic_tac_toe.dart
```

In VSCode select **ggwave Tic-Tac-Toe Demo** and press **F5** on each of two physical devices.

Suggested flow:

1. Device A taps **Host as X**. It starts listening and acoustically announces a short session.
2. Device B taps **Join as O**. It listens for the host, joins the session, and sends a ready packet.
3. Players take turns. Each move sends a compact versioned board-state packet over Audible Fast.
4. Duplicate/out-of-order packets from the same sender are ignored using sender + sequence tracking.
5. A received board update is accepted only when it is a valid one-cell transition for the remote player's mark.
6. **New round** resets the board while keeping the same acoustic session.

The game-specific wire format is intentionally located only under `example/lib/tic_tac_toe_protocol.dart`. It is not part of `ggwave-core`, `ggwave_dart`, or the public Flutter package API.

The current demo uses Audible Fast deliberately: its purpose is to show a small, reliable 1 ↔ 1 application protocol. Frequency/tuning validation remains the responsibility of the Android validation UI.
