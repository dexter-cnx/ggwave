# Platform support

## Support policy

| Platform | ggwave-core | ggwave_dart | ggwave_rs_flutter | ggwave-kotlin | Audio backend | Status |
|---|---|---|---|---|---|---|
| Android | Yes | Yes | Yes | Yes | native/CPAL (Flutter), Android app audio + JNI codec (Kotlin) | Tier 1 |
| iOS | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1 |
| macOS | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1 |
| Windows | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1 |
| Linux | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1 |
| Web | future WASM validation | Yes | Planned | N/A | Web Audio + WASM/JS | Tier 2 / planned |
| Fuchsia | not validated | Yes | Not planned yet | N/A | TBD | Unsupported |

`Yes` in this table means the source architecture and build target are implemented. Tier 1 promotion still requires the validation gate below to pass on representative hardware.

## Android and Kotlin

The native Kotlin binding is intentionally independent of Flutter:

```text
Kotlin/Java -> JNI -> dedicated Rust worker -> ggwave-core
```

Its Android project uses AGP 9 built-in Kotlin (`android.builtInKotlin=true`) and does not apply the legacy Kotlin Gradle plugin. This aligns with Flutter 3.47-era Android projects while remaining usable in ordinary native Android applications.

The first AAR targets `arm64-v8a`, `armeabi-v7a`, and `x86_64`.

## Why these five Flutter native platforms

They cover Flutter's mainstream mobile and desktop deployment targets and all have practical microphone/speaker paths. Keeping audio I/O outside `ggwave-core` prevents the codec crate from being coupled to CPAL, Flutter, or Android/JNI.

## Web

Web should not emulate the native implementation. A browser adapter should use `getUserMedia`, AudioWorklet/Web Audio for sample streaming, and either a WASM-compatible ggwave core or a small JS bridge. This keeps browser permission, latency and sample-rate behavior explicit.

## Validation gate

A platform is only promoted to fully validated support after encode/decode smoke tests, microphone capture, speaker playback, audible roundtrip, ultrasonic roundtrip on representative hardware, permissions, lifecycle/resume, and release-mode build all pass.

For Kotlin/Android, the release gate additionally requires all declared JNI ABIs to be packaged in the release AAR and an AGP 9 built-in Kotlin build to succeed.
