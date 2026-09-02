#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/packages/ggwave_flutter"

cd "$PKG"

if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
  echo 'flutter_rust_bridge_codegen is required (project pins 2.8.0).' >&2
  exit 2
fi

# FRB 2.8 uses its built-in Cargokit integration path. This package is a
# shareable Flutter plugin, and its Rust crate already lives at ./rust.
flutter_rust_bridge_codegen integrate \
  --template plugin \
  --no-enable-integration-test

flutter_rust_bridge_codegen generate

echo 'Native Flutter scaffold and FRB bindings generated.'
