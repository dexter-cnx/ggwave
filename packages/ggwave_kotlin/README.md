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

## Kotlin API

```kotlin
import io.github.dextercnx.ggwave.GgWave

GgWave.setUltrasonicFrequency(12_000f)

val waveform: FloatArray = GgWave.encode(
    data = "hello".encodeToByteArray(),
    protocolId = GgWave.PROTOCOL_ULTRASONIC_FAST,
    volume = 85,
)

// Feed microphone PCM converted to normalized mono Float samples.
val payload: ByteArray? = GgWave.decode(samples)
```

`GgWave` can be called from different JVM threads. Internally JNI forwards codec operations to one dedicated Rust worker thread because the upstream ggwave codec is intentionally non-Send/non-Sync.

## Build native libraries

Install the Android Rust targets and `cargo-ndk`, then run from the repository root:

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
./tool/build_kotlin_android.sh
```

This writes `libggwave_jni.so` into the AAR `jniLibs` directories for each supported ABI.

## Build the AAR

```bash
gradle -p packages/ggwave_kotlin assembleRelease
```

## Maven coordinates

Planned coordinates:

```text
io.github.dextercnx:ggwave-kotlin:1.2.0
```

Before a Maven Central release, configure signing and repository credentials outside the repository. Do not commit secrets.
