# Release guide

## Order

Publish in dependency order:

1. `ggwave-core` on crates.io
2. `ggwave_dart` on pub.dev
3. `ggwave_rs_flutter` on pub.dev
4. `ggwave-kotlin` to the configured Maven repository as `io.github.dextercnx:ggwave-kotlin`

The Flutter source directory is `packages/ggwave_flutter`, but the publishable Dart/Flutter package name is `ggwave_rs_flutter` because `ggwave_flutter` is already used by another package on pub.dev.

Run `./tool/release_check.sh` before the Rust/Dart/Flutter registry publishes and `./tool/release_kotlin_check.sh` before Maven publication.

Do not promote a platform from source implemented to build validated or hardware validated without the corresponding CI/build or physical-device evidence recorded in the repository documentation.

## Rust

```bash
cd crates/ggwave-core
cargo fmt --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo publish --dry-run
```

The core crate deliberately has no CPAL, Flutter, Android, microphone, or speaker dependency. `ultrasonic` and `dedup` are default features and can be disabled for smaller integrations.

## Dart

```bash
cd packages/ggwave_dart
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze
dart test
dart pub publish --dry-run
```

`ggwave_dart` is pure Dart. It owns protocol/tuning models, deduplication, and transport contracts but no Flutter APIs.

## Flutter

The package source is `packages/ggwave_flutter`; its publish name is `ggwave_rs_flutter`.

Flutter baseline:

```text
Flutter >= 3.47.0
Dart >= 3.12.0
flutter_rust_bridge = 2.8.0
```

Install the FRB 2.8 code generator as a Cargo binary, not as a Dart package:

```bash
cargo install flutter_rust_bridge_codegen --version 2.8.0 --locked
```

Generate the FRB/Cargokit integration before validation:

```bash
./tool/bootstrap_flutter_native.sh
```

The native Tier 1 targets are Android, iOS, macOS, Windows, and Linux. Each target must complete its own build validation before release support is claimed. Physical microphone/speaker validation is a separate gate.

Web is not claimed as native-CPAL support in 1.2.0. It requires a separate Web Audio/AudioWorklet plus WASM/JS backend.

Before publishing:

```bash
cd packages/ggwave_flutter
flutter pub get
dart format --output=none --set-exit-if-changed lib test example/lib
flutter analyze
flutter test
flutter pub publish --dry-run
```

Local monorepo `pubspec_overrides.yaml` files are development-only glue for the unpublished `ggwave_dart` dependency and must not change the public dependency graph.

## Kotlin / Android

The Maven artifact is intended to be:

```text
io.github.dextercnx:ggwave-kotlin
```

Android baseline:

```text
minSdk 23
compileSdk 36
AGP 9.0.0
Gradle >= 9.1
android.builtInKotlin=true
```

The AAR packages Rust JNI libraries for:

```text
arm64-v8a
armeabi-v7a
x86_64
```

Run:

```bash
./tool/release_kotlin_check.sh
```

Kotlin/Android build validation is established by `Kotlin Android #18`, run `33600474569`. Physical acoustic validation remains a separate pending release gate; see `docs/ANDROID_VALIDATION.md`.

## Final release gate

Before creating release tags or publishing registries, verify all of the following:

- package versions and changelogs are consistent;
- `README.md`, `CODE_WALKTHROUGH.md`, `docs/PLATFORMS.md`, and validation docs match the actual evidence;
- `ggwave-core` has no application-specific or platform-audio ownership;
- no Bingo, QR schema, pairing format, game ID, device ID, or game-state logic has entered the universal packages;
- Rust, Dart, Flutter, and Kotlin release checks pass;
- every platform claim distinguishes source implementation, build validation, and hardware validation;
- registry dry-runs pass before the first real publish;
- release order remains `ggwave-core` -> `ggwave_dart` -> `ggwave_rs_flutter` -> `ggwave-kotlin`.
