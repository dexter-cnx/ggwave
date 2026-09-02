# Release guide

Publish in dependency order:

1. `ggwave-mobile` 1.2.0 → crates.io.
2. `ggwave_dart` 1.2.0 → pub.dev.
3. Regenerate FRB glue in `ggwave_flutter`, validate it, then publish `ggwave_flutter` 1.2.0 → pub.dev.

Registry names are first-come-first-served. Re-check `ggwave-mobile`, `ggwave_dart`, and `ggwave_flutter` immediately before the first publish. If a name is taken, rename consistently before publishing any layer.

## Validation

Run:

```bash
./tool/release_check.sh
```

The script intentionally refuses to claim the Flutter package is publishable while FRB glue is still a placeholder.

## Rust release

The Rust crate uses `ggwave-rs = "0.1.1"` from crates.io rather than a moving Git branch so the published dependency graph is reproducible.

From `crates/ggwave-mobile`:

```bash
cargo test
cargo publish --dry-run
cargo publish
```

## Dart release

From `packages/ggwave_dart`:

```bash
dart pub get
dart analyze
dart test
dart pub publish --dry-run
dart pub publish
```

## Flutter release

From `packages/ggwave_flutter`:

```bash
flutter pub get
cargo install flutter_rust_bridge_codegen --version 2.8.0
flutter_rust_bridge_codegen generate
flutter analyze
flutter test
dart pub publish --dry-run
dart pub publish
```
