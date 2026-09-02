#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo '1/3 crates.io: ggwave-mobile'
(cd "$ROOT/crates/ggwave-mobile" && cargo publish)
echo '2/3 pub.dev: ggwave_dart'
(cd "$ROOT/packages/ggwave_dart" && dart pub publish)
echo '3/3 pub.dev: ggwave_flutter'
(cd "$ROOT/packages/ggwave_flutter" && dart pub publish)
