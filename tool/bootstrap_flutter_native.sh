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

flutter_rust_bridge_codegen integrate \
  --template plugin \
  --no-integration-test

if [ -f android/build.gradle ]; then
  sed -i.bak -E \
    -e 's/compileSdkVersion[[:space:]]+33/compileSdkVersion 36/g' \
    -e 's/compileSdk[[:space:]]+33/compileSdk 36/g' \
    android/build.gradle
  rm -f android/build.gradle.bak
fi
if [ -f android/build.gradle.kts ]; then
  sed -i.bak -E \
    -e 's/compileSdk[[:space:]]*=[[:space:]]*33/compileSdk = 36/g' \
    android/build.gradle.kts
  rm -f android/build.gradle.kts.bak
fi
if grep -R -E 'compileSdk(Version)?[[:space:]=]+33' android --include='*.gradle' --include='*.gradle.kts'; then
  echo 'FRB Android scaffold still contains compileSdk 33 after normalization.' >&2
  exit 1
fi

rm -f rust/src/api/simple.rs
rm -f lib/src/rust/api/simple.dart
rm -rf test_driver integration_test

restore_package_files
trap - EXIT
flutter pub get
flutter_rust_bridge_codegen generate

echo 'Native Flutter scaffold and FRB bindings generated with Android compileSdk 36, without changing the package manifest/public barrel or retaining template demo files.'
