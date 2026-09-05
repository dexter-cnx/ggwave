#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/packages/ggwave_flutter"

cd "$PKG"

if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
  echo 'flutter_rust_bridge_codegen is required (project pins 2.8.0).' >&2
  exit 2
fi

original_pubspec="$(mktemp)"
original_barrel="$(mktemp)"
cp pubspec.yaml "$original_pubspec"
cp lib/ggwave_rs_flutter.dart "$original_barrel"
restore_package_files() {
  cp "$original_pubspec" pubspec.yaml
  cp "$original_barrel" lib/ggwave_rs_flutter.dart
  rm -f "$original_pubspec" "$original_barrel"
}
trap restore_package_files EXIT

# FRB 2.8.0 distributions in the wild expose one of two spellings for
# disabling the integration-test template. Detect the actual CLI contract
# instead of assuming one spelling so local VSCode and CI stay reproducible.
integrate_help="$(flutter_rust_bridge_codegen integrate --help 2>&1)"
if grep -q -- '--no-enable-integration-test' <<<"$integrate_help"; then
  no_integration_test_flag='--no-enable-integration-test'
elif grep -q -- '--no-integration-test' <<<"$integrate_help"; then
  no_integration_test_flag='--no-integration-test'
else
  echo 'Unable to determine the FRB integrate flag for disabling integration tests.' >&2
  echo "$integrate_help" >&2
  exit 2
fi

echo "Using FRB integrate flag: $no_integration_test_flag"
flutter_rust_bridge_codegen integrate \
  --template plugin \
  "$no_integration_test_flag"

# Normalize only generated Android compatibility knobs. Flutter 3.47 / AGP
# builds Kotlin with JVM 17, while some generated plugin templates still leave
# Java source/target compatibility at 1.8. Gradle rejects that mismatch.
if [ -f android/build.gradle ]; then
  sed -i.bak -E \
    -e 's/compileSdkVersion[[:space:]]+33/compileSdkVersion 36/g' \
    -e 's/compileSdk[[:space:]]+33/compileSdk 36/g' \
    -e 's/JavaVersion\.VERSION_1_8/JavaVersion.VERSION_17/g' \
    android/build.gradle
  rm -f android/build.gradle.bak
fi
if [ -f android/build.gradle.kts ]; then
  sed -i.bak -E \
    -e 's/compileSdk[[:space:]]*=[[:space:]]*33/compileSdk = 36/g' \
    -e 's/JavaVersion\.VERSION_1_8/JavaVersion.VERSION_17/g' \
    android/build.gradle.kts
  rm -f android/build.gradle.kts.bak
fi

# CPAL's Android backend reads ndk-context. Because the Rust library is loaded
# from Dart FFI/native assets instead of an Android Activity runtime, install a
# tiny FlutterPlugin bootstrap that passes applicationContext to Rust before
# any CPAL call is made.
android_plugin_src="$ROOT/tool/android/GgwaveRsFlutterPlugin.kt"
android_plugin_dst='android/src/main/kotlin/com/dextercnx/ggwave/GgwaveRsFlutterPlugin.kt'
if [ ! -f "$android_plugin_src" ]; then
  echo "Missing Android context bootstrap template: $android_plugin_src" >&2
  exit 1
fi
mkdir -p "$(dirname "$android_plugin_dst")"
cp "$android_plugin_src" "$android_plugin_dst"

# AGP no longer allows Android library namespace to be declared with the
# manifest package attribute. The generated plugin build.gradle(.kts) owns the
# namespace, so strip only this legacy attribute from the generated manifest.
manifest='android/src/main/AndroidManifest.xml'
if [ -f "$manifest" ]; then
  python3 - "$manifest" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
text = path.read_text()
text = re.sub(r'\s+package="[^"]+"', '', text, count=1)
path.write_text(text)
PY
fi

if grep -R -E 'compileSdk(Version)?[[:space:]=]+33' android --include='*.gradle' --include='*.gradle.kts'; then
  echo 'FRB Android scaffold still contains compileSdk 33 after normalization.' >&2
  exit 1
fi
if grep -R -F 'JavaVersion.VERSION_1_8' android --include='*.gradle' --include='*.gradle.kts'; then
  echo 'FRB Android scaffold still contains Java 1.8 after JVM 17 normalization.' >&2
  exit 1
fi
if [ -f "$manifest" ] && grep -q -E '<manifest[^>]+package=' "$manifest"; then
  echo 'FRB Android library manifest still contains a legacy package attribute.' >&2
  exit 1
fi
if [ ! -f "$android_plugin_dst" ]; then
  echo 'Android native-context Flutter plugin was not installed.' >&2
  exit 1
fi

rm -f rust/src/api/simple.rs
rm -f lib/src/rust/api/simple.dart
rm -rf test_driver integration_test

restore_package_files
trap - EXIT
flutter pub get
flutter_rust_bridge_codegen generate

echo 'Native Flutter scaffold and FRB bindings generated with Android compileSdk 36, JVM 17, AGP-compatible manifest, and CPAL Android context bootstrap.'
