#!/usr/bin/env bash
# Builds what the Android app (#3) links against: the passport's native
# library for each ABI the app ships, and the Kotlin bindings generated from
# that same library, so the two can never drift apart.
#
# Needs: ANDROID_NDK_HOME, cargo-ndk, and the Rust targets
# aarch64-linux-android and x86_64-linux-android.
#
# Usage: scripts/build_android_libs.sh [app-src-main-dir]
#   default: android/app/src/main  ->  jniLibs/<abi>/  and  kotlin/
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/android/app/src/main}"
min_sdk=26

cd "$root"
cargo ndk --platform "$min_sdk" -t arm64-v8a -t x86_64 -o "$out/jniLibs" \
    build -p macula-passport-ffi --release --locked
cargo run -p macula-passport-ffi --release --locked --bin uniffi-bindgen -- generate \
    --library target/aarch64-linux-android/release/libmacula_passport_ffi.so \
    --language kotlin --out-dir "$out/kotlin" --no-format
