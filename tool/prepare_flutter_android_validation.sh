#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DART_PKG="$ROOT/packages/ggwave_dart"
PKG="$ROOT/packages/ggwave_flutter"
EXAMPLE="$PKG/example"
FRB_VERSION="2.8.0"

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

install_frb_codegen() {
  echo "Installing flutter_rust_bridge_codegen $FRB_VERSION..."
  cargo install flutter_rust_bridge_codegen --version "$FRB_VERSION" --locked --force
}

if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
  install_frb_codegen
else
  installed_frb="$(flutter_rust_bridge_codegen --version 2>/dev/null || true)"
  if [[ "$installed_frb" != *"$FRB_VERSION"* ]]; then
    echo "Found incompatible FRB codegen: ${installed_frb:-unknown}. Expected $FRB_VERSION."
    install_frb_codegen
  fi
fi

echo "Using $(flutter_rust_bridge_codegen --version)"

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

echo 'Resolving ggwave_dart dependencies...'
(cd "$DART_PKG" && dart pub get)

ensure_plugin_android_scaffold
bash "$ROOT/tool/bootstrap_flutter_native.sh"
ensure_plugin_android_scaffold

if [ ! -f "$PKG/android/build.gradle" ] && [ ! -f "$PKG/android/build.gradle.kts" ]; then
  echo "Flutter plugin Android scaffold is incomplete: $PKG/android" >&2
  exit 1
fi

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

# The Android runner is ephemeral. `flutter create` also writes a stock widget
# test that references MyApp and an analysis_options.yaml that depends on
# flutter_lints. This repository restores its own main.dart/pubspec.yaml, so
# retaining those template files only creates false VSCode analyzer errors.
rm -f test/widget_test.dart analysis_options.yaml
rmdir test 2>/dev/null || true

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
