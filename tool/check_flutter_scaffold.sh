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

if [[ "$missing" -ne 0 ]]; then
  echo 'Run ./tool/bootstrap_flutter_native.sh and commit the generated scaffold.' >&2
  exit 2
fi

echo 'Flutter native scaffold is present for Android/iOS/macOS/Windows/Linux.'
