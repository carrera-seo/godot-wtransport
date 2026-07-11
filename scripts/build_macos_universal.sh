#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
godot_cpp_source="${GWT_GODOT_CPP_SOURCE:-}"
deployment_target="${GWT_MACOS_DEPLOYMENT_TARGET:-12.0}"

cmake_source_args=()
if [[ -n "$godot_cpp_source" ]]; then
    cmake_source_args+=("-DGWT_GODOT_CPP_SOURCE=$godot_cpp_source")
fi

for flavor in Debug Release; do
    flavor_lower="$(printf '%s' "$flavor" | tr '[:upper:]' '[:lower:]')"
    slices=()
    for arch in arm64 x86_64; do
        case "$arch" in
            arm64) rust_target="aarch64-apple-darwin" ;;
            x86_64) rust_target="x86_64-apple-darwin" ;;
        esac
        build_dir="$root/build-macos-$flavor_lower-$arch"
        rustup target add "$rust_target"
        cmake -S "$root" -B "$build_dir" \
            -DCMAKE_BUILD_TYPE="$flavor" \
            -DCMAKE_OSX_ARCHITECTURES="$arch" \
            -DGWT_BUILD_FFI_HARNESS=OFF \
            -DGWT_RUST_TARGET="$rust_target" \
            -DGWT_OUTPUT_ARCH="$arch" \
            -DGWT_MACOS_DEPLOYMENT_TARGET="$deployment_target" \
            "${cmake_source_args[@]}"
        cmake --build "$build_dir" --parallel
        slices+=("$root/demo/addons/godot_wtransport/bin/libgodot_wtransport.macos.$flavor_lower.$arch.dylib")
    done
    lipo -create "${slices[@]}" \
        -output "$root/demo/addons/godot_wtransport/bin/libgodot_wtransport.macos.$flavor_lower.universal.dylib"
    lipo "$root/demo/addons/godot_wtransport/bin/libgodot_wtransport.macos.$flavor_lower.universal.dylib" \
        -verify_arch arm64 x86_64
    rm -f "${slices[@]}"
done
