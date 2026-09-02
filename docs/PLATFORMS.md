# Platform support

## Support policy

| Platform | ggwave-core | ggwave_dart | ggwave_rs_flutter | ggwave-kotlin | Audio backend | Status |
|---|---|---|---|---|---|---|
| Android | Yes | Yes | Yes | Yes | native/CPAL (Flutter), AudioRecord/AudioTrack + JNI codec (Kotlin) | Tier 1; Kotlin build validated, hardware pending; Flutter build validated, hardware pending |
| iOS | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1; source implemented, build validation pending |
| macOS | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1; source implemented, build validation pending |
| Windows | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1; source implemented, build validation pending |
| Linux | Yes | Yes | Yes | N/A | native/CPAL via Rust | Tier 1; source implemented, build validation pending |
| Web | future WASM validation | Yes | Planned | N/A | Web Audio + WASM/JS | Tier 2 / planned |
| Fuchsia | not validated | Yes | Not planned yet | N/A | TBD | Unsupported |

`Yes` in this table means the source architecture and build target are implemented. It does not by itself mean that a platform is build validated or hardware validated. Tier 1 promotion still requires the validation gate below to pass on representative hardware.

## Android and Kotlin

The native Kotlin binding is intentionally independent of Flutter:

```text
Kotlin/Java -> AudioRecord/AudioTrack -> JNI -> dedicated Rust worker -> ggwave-core
```

Its Android project uses AGP 9.0.0 built-in Kotlin (`android.builtInKotlin=true`) with Gradle 9.1+ and does not apply the legacy Kotlin Gradle plugin. This aligns with Flutter 3.47-era Android projects while remaining usable in ordinary native Android applications.

The AAR targets `arm64-v8a`, `armeabi-v7a`, and `x86_64`.

Kotlin/Android is build validated by GitHub Actions workflow `Kotlin Android #18`, run `33600474569` (2026-09-02). The gate passed Rust formatting/check/clippy, all three JNI ABI builds, AGP 9 built-in Kotlin, release AAR assembly, Maven POM generation, standalone validation-app compilation against the public Kotlin API, and artifact upload.

Artifacts from that run:

- `ggwave-kotlin-release`, SHA-256 `0b34957c62ac752d1856584bf543007b53d59e981bfc4f1e3c5c2721d2c3606f`;
- `ggwave-kotlin-validation-apk`, SHA-256 `6ca205abe5c17150be07e1b59d1c6f23b41b96b9adb45f58728c7750cb737be1`.

Physical-device acoustic validation is still pending, so this evidence supports **build validated**, not **hardware validated**.

## Android and Flutter

The Flutter binding uses the same `ggwave-core` through FRB and keeps the Flutter-facing packaging/audio lifecycle in `packages/ggwave_flutter`.

Flutter/Android is build validated by GitHub Actions workflow `Flutter Android #25`, run `33615793479` (2026-09-02). The gate passed Flutter 3.47.0 setup, FRB 2.8.0 codegen installation, monorepo dependency resolution, FRB/Cargokit integration, binding generation, Dart format/analyze/test, Flutter format/analyze/test, Android example scaffold generation, debug APK build, and artifact upload.

FRB 2.8's plugin template initially emits `compileSdk 33`; the repository bootstrap normalizes that generated scaffold to `compileSdk 36` and verifies the old value is gone before continuing. This keeps the generated Android plugin compatible with Flutter 3.47's AndroidX dependencies without changing minSdk or targetSdk policy as a side effect.

Artifact from run `33615793479`:

- `ggwave-rs-flutter-android-example`, workflow artifact digest `sha256:76c6b2a47dc78e52aa98ecec6a3620cf1e692f746dcee4b8fd4729826240aaa0`.

This establishes **Flutter Android build validation only**. Acoustic hardware validation is still pending.

## Why these five Flutter native platforms

They cover Flutter's mainstream mobile and desktop deployment targets and all have practical microphone/speaker paths. Keeping audio I/O outside `ggwave-core` prevents the codec crate from being coupled to CPAL, Flutter, or Android/JNI.

## Web

Web should not emulate the native implementation. A browser adapter should use `getUserMedia`, AudioWorklet/Web Audio for sample streaming, and either a WASM-compatible ggwave core or a small JS bridge. This keeps browser permission, latency and sample-rate behavior explicit.

## Validation gate

A platform is only promoted to fully validated support after encode/decode smoke tests, microphone capture, speaker playback, audible roundtrip, ultrasonic roundtrip on representative hardware, permissions, lifecycle/resume, and release-mode build all pass.

For Kotlin/Android, the build portion of that gate additionally requires all declared JNI ABIs to be packaged in the release AAR and an AGP 9 built-in Kotlin build to succeed. That build portion is validated by run `33600474569`; hardware acceptance remains outstanding.

For Flutter/Android, the build portion is validated by run `33615793479`; physical acoustic acceptance remains outstanding. For iOS, macOS, Windows, and Linux, source implementation is present but platform build validation is still pending.
