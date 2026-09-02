#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/packages/ggwave_flutter"

cd "$PKG"

if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
  echo 'flutter_rust_bridge_codegen is required (project pins 2.8.0).' >&2
  exit 2
fi

# FRB 2.x uses Cargokit as its broadly compatible integration backend. This
# command creates/refreshes the native build scaffold for Flutter targets.
flutter_rust_bridge_codegen integrate \
  --integration-backend cargokit \
  --platforms android,ios,macos,windows,linux

flutter_rust_bridge_codegen generate

echo 'Native Flutter scaffold and FRB bindings generated.'
