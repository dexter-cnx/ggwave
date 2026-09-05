#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/packages/ggwave_flutter"

required=(android ios macos windows linux)
missing=0
for platform in "${required[@]}"; do
  if [[ ! -d "$PKG/$platform" ]]; then
    echo "MISSING: packages/ggwave_flutter/$platform" >&2
    missing=1
  fi
done

if grep -Rqs 'GENERATED FILE PLACEHOLDER' "$PKG/lib/src/rust"; then
  echo 'MISSING: real flutter_rust_bridge generated Dart bindings' >&2
  missing=1
fi

android_context_plugin="$PKG/android/src/main/kotlin/com/dextercnx/ggwave/GgwaveRsFlutterPlugin.kt"
if [[ ! -f "$android_context_plugin" ]]; then
  echo 'MISSING: Android CPAL/ndk-context bootstrap plugin' >&2
  missing=1
fi

if [[ "$missing" -ne 0 ]]; then
  echo 'Run make bootstrap (or ./tool/bootstrap_flutter_native.sh) to recreate generated native scaffolds.' >&2
  exit 2
fi

echo 'Flutter native scaffold is present and Android CPAL context bootstrap is installed.'
