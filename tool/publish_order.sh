#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo 'Publish order:'
echo "1) cd $ROOT/crates/ggwave-core && cargo publish --dry-run && cargo publish"
echo "2) cd $ROOT/packages/ggwave_dart && dart pub publish --dry-run && dart pub publish"
echo "3) generate FRB, validate native targets, then cd $ROOT/packages/ggwave_flutter && flutter pub publish --dry-run && flutter pub publish"
echo "4) run $ROOT/tool/release_kotlin_check.sh, then publish io.github.dextercnx:ggwave-kotlin to the configured Maven repository"
