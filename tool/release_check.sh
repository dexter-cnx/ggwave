#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo '== Rust workspace =='
(cd "$ROOT" && cargo fmt --check --all && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings)

echo '== Rust crate publish dry-run =='
(cd "$ROOT/crates/ggwave-core" && cargo publish --dry-run)

echo '== Dart package =='
(cd "$ROOT/packages/ggwave_dart" && dart pub get && dart format --output=none --set-exit-if-changed . && dart analyze && dart test && dart pub publish --dry-run)

echo '== Flutter native scaffold =='
"$ROOT/tool/check_flutter_scaffold.sh"

echo '== Flutter package =='
(cd "$ROOT/packages/ggwave_flutter" && flutter pub get && dart format --output=none --set-exit-if-changed lib test && flutter analyze && flutter test && dart pub publish --dry-run)

echo 'All publish dry-runs passed.'
