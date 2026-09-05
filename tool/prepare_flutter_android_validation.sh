#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DART_PKG="$ROOT/packages/ggwave_dart"
PKG="$ROOT/packages/ggwave_flutter"
EXAMPLE="$PKG/example"

if ! command -v flutter >/dev/null 2>&1; then
  echo 'Flutter is required and must be available on PATH.' >&2
  exit 2
fi

if ! command -v dart >/dev/null 2>&1; then
  echo 'Dart is required and must be available on PATH.' >&2
  exit 2
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo 'Rust/Cargo is required and must be available on PATH.' >&2
  exit 2
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo 'python3 is required to patch the generated Android manifest.' >&2
  exit 2
fi

if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
  echo 'Installing flutter_rust_bridge_codegen 2.8.0...'
  cargo install flutter_rust_bridge_codegen --version 2.8.0 --locked
fi

ensure_plugin_android_scaffold() {
  if [ -d "$PKG/android" ]; then
    return
  fi

  echo 'Creating local Flutter plugin Android scaffold...'
  local tmp
  tmp="$(mktemp -d)"

  flutter create \
    --template=plugin \
    --platforms=android \
    --org io.github.dextercnx \
    --project-name ggwave_rs_flutter \
    "$tmp/ggwave_rs_flutter"

  if [ ! -d "$tmp/ggwave_rs_flutter/android" ]; then
    rm -rf "$tmp"
    echo 'Flutter did not create the expected plugin Android scaffold.' >&2
    exit 1
  fi

  cp -R "$tmp/ggwave_rs_flutter/android" "$PKG/android"
  rm -rf "$tmp"
}

# Resolve the pure-Dart package independently so VSCode's analyzer has a valid
# package_config.json for its example/tests, not just for Flutter consumers.
echo 'Resolving ggwave_dart dependencies...'
(cd "$DART_PKG" && dart pub get)

# The example depends on ggwave_rs_flutter as a local plugin. Native platform
# scaffolds are intentionally generated locally rather than committed.
ensure_plugin_android_scaffold

bash "$ROOT/tool/bootstrap_flutter_native.sh"

# FRB integration is allowed to regenerate platform state. Re-check after the
# bootstrap instead of assuming the pre-bootstrap scaffold survived unchanged.
ensure_plugin_android_scaffold

if [ ! -f "$PKG/android/build.gradle" ] && [ ! -f "$PKG/android/build.gradle.kts" ]; then
  echo "Flutter plugin Android scaffold is incomplete: $PKG/android" >&2
  exit 1
fi

# Refresh Flutter package resolution after FRB generation for both command-line
# builds and VSCode's analysis server.
echo 'Resolving ggwave_rs_flutter dependencies...'
(cd "$PKG" && flutter pub get)

cd "$EXAMPLE"
if [ ! -d android ]; then
  original_pubspec="$(mktemp)"
  original_main="$(mktemp)"
  cp pubspec.yaml "$original_pubspec"
  cp lib/main.dart "$original_main"

  flutter create \
    --platforms=android \
    --project-name ggwave_rs_flutter_example \
    .

  cp "$original_pubspec" pubspec.yaml
  cp "$original_main" lib/main.dart
  rm -f "$original_pubspec" "$original_main"
fi

manifest="android/app/src/main/AndroidManifest.xml"
python3 - "$manifest" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
permission = '<uses-permission android:name="android.permission.RECORD_AUDIO" />'
if permission not in text:
    marker = '<manifest xmlns:android="http://schemas.android.com/apk/res/android">'
    if marker not in text:
        raise SystemExit(f'Unexpected AndroidManifest structure: {path}')
    text = text.replace(marker, marker + '\n    ' + permission, 1)
    path.write_text(text)
PY

flutter pub get

echo
echo 'Android validation example is ready.'
echo "Plugin scaffold: $PKG/android"
echo 'Connect an Android device with USB debugging enabled, then run the VSCode launch configuration:'
echo '  ggwave Android Validation'
