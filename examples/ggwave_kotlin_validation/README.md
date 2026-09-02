# ggwave Kotlin Android validation app

Small native Android app for physical-device acceptance of `ggwave-kotlin`.

## Build

From the repository root with Android SDK 36 and Gradle 9.1+ installed:

```bash
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
bash ./tool/build_kotlin_android.sh
gradle -p examples/ggwave_kotlin_validation :app:assembleDebug
```

Install the debug APK on two physical Android devices.

## Two-device procedure

Use device A as transmitter and device B as receiver, then reverse direction.

1. On both devices grant microphone permission.
2. On the receiving device tap **Start listening**.
3. On the transmitting device send **Audible** ten times and record received count.
4. Repeat for **12 kHz**, **15 kHz**, and **18 kHz** ultrasonic modes.
5. Test at 0.5 m, 1 m, and 2 m where practical.
6. Repeat once with each device rotated 180 degrees or otherwise changing speaker/microphone orientation.
7. Background and resume the receiving app, start listening again, and confirm packets still arrive.
8. Perform ten start/listen/stop cycles without restarting the app.
9. Reverse transmitter/receiver roles and repeat.

Payloads are emitted in this form:

```text
GGWAVE_VALIDATE:<audible|frequencyHz>:<sequence>
```

The screen reports permission state, listening state, sent count, received count, and the latest decoded payload.

## Record evidence

For each device pair record:

- device model;
- Android version;
- direction (A -> B / B -> A);
- mode/frequency;
- distance;
- orientation;
- sent count;
- received count;
- audible artifacts or unexpected behavior;
- background/resume result;
- repeated start/stop result.

Do not mark Android acoustic support as hardware validated until the acceptance criteria in `docs/ANDROID_VALIDATION.md` are satisfied.
