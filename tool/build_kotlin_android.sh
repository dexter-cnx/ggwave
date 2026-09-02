#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/packages/ggwave_kotlin/src/main/jniLibs"

command -v cargo >/dev/null || { echo 'cargo is required' >&2; exit 1; }
command -v cargo-ndk >/dev/null || { echo 'cargo-ndk is required: cargo install cargo-ndk' >&2; exit 1; }

rm -rf "$OUT"
mkdir -p "$OUT"

cd "$ROOT"
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -o "$OUT" \
  build --release -p ggwave-jni

find "$OUT" -name 'libggwave_jni.so' -print
