#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo '== Rust core =='
(cd "$ROOT/crates/ggwave-core" && cargo fmt --check && cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings && cargo publish --dry-run)

echo '== Dart package =='
(cd "$ROOT/packages/ggwave_dart" && dart pub get && dart format --output=none --set-exit-if-changed . && dart analyze && dart test && dart pub publish --dry-run)

echo '== Flutter package =='
if grep -q 'GENERATED FILE PLACEHOLDER' "$ROOT/packages/ggwave_flutter/lib/src/rust/api.dart"; then
  echo 'ERROR: FRB generated Dart is still a placeholder. Run flutter_rust_bridge_codegen generate.' >&2
  exit 2
fi
(cd "$ROOT/packages/ggwave_flutter" && flutter pub get && dart format --output=none --set-exit-if-changed lib test && flutter analyze && flutter test && dart pub publish --dry-run)

echo 'All publish dry-runs passed.'
