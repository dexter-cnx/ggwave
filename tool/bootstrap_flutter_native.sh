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
#
# FRB 2.8 currently mutates pubspec.yaml during `integrate` by adding an
# integration_test SDK dependency even when integration tests are disabled.
# Preserve the package manifest so scaffold generation cannot change the
# public dependency graph.
original_pubspec="$(mktemp)"
cp pubspec.yaml "$original_pubspec"
restore_pubspec() {
  cp "$original_pubspec" pubspec.yaml
  rm -f "$original_pubspec"
}
trap restore_pubspec EXIT

flutter_rust_bridge_codegen integrate \
  --template plugin \
  --no-enable-integration-test

restore_pubspec
trap - EXIT
flutter pub get
flutter_rust_bridge_codegen generate

echo 'Native Flutter scaffold and FRB bindings generated without changing pubspec.yaml.'
