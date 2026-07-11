#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
godot_cpp="${GWT_GODOT_CPP_SOURCE:-}"

if [[ "${GWT_CONFIRM_IOS_BUILD:-}" != "yes" ]]; then
    echo "iOS build is intentionally deferred. Set GWT_CONFIRM_IOS_BUILD=yes only when explicitly requested." >&2
    exit 2
fi
if [[ -z "$godot_cpp" ]]; then
    echo "GWT_GODOT_CPP_SOURCE is required" >&2
    exit 2
fi

echo "This command prepares device-only iOS static archives; it does not run or test an iOS app."
rustup target add aarch64-apple-ios
for flavor in Debug Release; do
    lower="$(printf '%s' "$flavor" | tr '[:upper:]' '[:lower:]')"
    build_dir="$root/build-ios-$lower-arm64"
    cmake -S "$root" -B "$build_dir" -G Xcode \
        -DCMAKE_BUILD_TYPE="$flavor" \
        -DCMAKE_SYSTEM_NAME=iOS \
        -DCMAKE_OSX_ARCHITECTURES=arm64 \
        -DCMAKE_OSX_DEPLOYMENT_TARGET=13.0 \
        -DGWT_BUILD_FFI_HARNESS=OFF \
        -DGWT_GODOT_CPP_SOURCE="$godot_cpp" \
        -DGWT_RUST_TARGET=aarch64-apple-ios \
        -DGWT_OUTPUT_ARCH=arm64
    cmake --build "$build_dir" --config "$flavor" --parallel
done

echo "Static archive integration into the Godot iOS export must be completed during the explicitly deferred iOS build/test task."
