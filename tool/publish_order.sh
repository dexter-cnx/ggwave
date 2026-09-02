#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo 'Publish order:'
echo "1) cd $ROOT/crates/ggwave-core && cargo publish --dry-run && cargo publish"
echo "2) cd $ROOT/packages/ggwave_dart && dart pub publish --dry-run && dart pub publish"
echo "3) generate FRB, validate native targets, then cd $ROOT/packages/ggwave_flutter && dart pub publish --dry-run && dart pub publish"
