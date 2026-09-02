#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

command -v cargo >/dev/null || { echo 'cargo is required' >&2; exit 1; }
command -v cargo-ndk >/dev/null || { echo 'cargo-ndk is required' >&2; exit 1; }
command -v gradle >/dev/null || { echo 'Gradle 9+ is required' >&2; exit 1; }
: "${ANDROID_NDK_HOME:?ANDROID_NDK_HOME must point to an installed Android NDK}"

echo '== JNI host compile/lints =='
(cd "$ROOT" && cargo fmt --check --all && cargo check -p ggwave-jni && cargo clippy -p ggwave-jni -- -D warnings)

echo '== Android Rust ABIs =='
bash "$ROOT/tool/build_kotlin_android.sh"
for abi in arm64-v8a armeabi-v7a x86_64; do
  test -f "$ROOT/packages/ggwave_kotlin/src/main/jniLibs/$abi/libggwave_jni.so" || {
    echo "missing JNI library for $abi" >&2
    exit 2
  }
done

echo '== AGP 9 built-in Kotlin AAR =='
gradle -p "$ROOT/packages/ggwave_kotlin" clean assembleRelease generatePomFileForReleasePublication

AAR_COUNT="$(find "$ROOT/packages/ggwave_kotlin/build/outputs/aar" -maxdepth 1 -name '*-release.aar' -type f | wc -l | tr -d ' ')"
if [ "$AAR_COUNT" -lt 1 ]; then
  echo 'release AAR was not produced' >&2
  exit 3
fi

echo 'Kotlin/Android release gate passed.'
