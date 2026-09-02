# ggwave-kotlin

Native Kotlin/Android binding for the universal `ggwave-core` Rust crate.

This package does not depend on Flutter and does not contain application-specific protocols such as Bingo or QR payloads.

## Compatibility

- Android minSdk 23
- compileSdk 36
- AGP 9+
- built-in Kotlin enabled (`android.builtInKotlin=true`)
- arm64-v8a, armeabi-v7a, x86_64 native ABIs

The AGP 9 built-in Kotlin layout is intentionally compatible with the Flutter 3.47 Android toolchain direction. The library itself is usable from a normal native Android/Kotlin project without Flutter.

## High-level Android API

The AAR declares `android.permission.RECORD_AUDIO`, but the host application must still request the runtime permission before listening.

```kotlin
import io.github.dextercnx.ggwave.GgWave
import io.github.dextercnx.ggwave.GgWaveAudio

GgWave.setUltrasonicFrequency(12_000f)

// After RECORD_AUDIO runtime permission has been granted:
GgWaveAudio.startListening { payload ->
    println(payload.decodeToString())
}

GgWaveAudio.send(
    data = "hello".encodeToByteArray(),
    protocolId = GgWave.PROTOCOL_ULTRASONIC_FAST,
    volume = 85,
)

// Later, e.g. onStop/onDestroy:
GgWaveAudio.stopListening()
```

`GgWaveAudio` uses 48 kHz mono PCM float audio. Capture uses `AudioRecord`; finite ggwave packets use a static `AudioTrack` and the playback worker waits for the entire waveform before releasing it.

The receive callback executes on the library capture thread. Dispatch to the main thread if the callback updates Android UI.

## Low-level codec API

Use the lower-level API when your application already owns audio I/O:

```kotlin
val waveform: FloatArray = GgWave.encode(
    data = "hello".encodeToByteArray(),
    protocolId = GgWave.PROTOCOL_ULTRASONIC_FAST,
    volume = 85,
)

val payload: ByteArray? = GgWave.decode(monoPcmFloatSamples)
```

`GgWave` can be called from different JVM threads. Internally JNI forwards codec operations to one dedicated Rust worker thread because the upstream ggwave codec is intentionally non-Send/non-Sync.

## Build native libraries

Install the Android Rust targets and `cargo-ndk`, then run from the repository root:

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
bash ./tool/build_kotlin_android.sh
```

This writes `libggwave_jni.so` into the AAR `jniLibs` directories for each supported ABI.

## Validate the Android artifact

```bash
bash ./tool/release_kotlin_check.sh
```

The gate checks Rust formatting/lints, cross-compiles all supported ABIs, builds the AGP 9 AAR and generates the Maven POM.

## Maven coordinates

Planned coordinates:

```text
io.github.dextercnx:ggwave-kotlin:1.2.0
```

Before a Maven Central release, configure signing and repository credentials outside the repository. Do not commit secrets.
