#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
case "$(uname -s)" in
    Darwin)
        exec "$root/scripts/build_macos_universal.sh"
        ;;
    Linux)
        platform="linux"
        rust_target="x86_64-unknown-linux-gnu"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        platform="windows"
        rust_target="x86_64-pc-windows-msvc"
        ;;
    *)
        echo "Unsupported desktop platform" >&2
        exit 2
        ;;
esac

rustup target add "$rust_target"
generator_args=(-G Ninja)
cmake_source_args=()
if [[ -n "${GWT_GODOT_CPP_SOURCE:-}" ]]; then
    cmake_source_args+=("-DGWT_GODOT_CPP_SOURCE=${GWT_GODOT_CPP_SOURCE}")
fi
for flavor in Debug Release; do
    build_dir="$root/build-$platform-$(printf '%s' "$flavor" | tr '[:upper:]' '[:lower:]')"
    cmake -S "$root" -B "$build_dir" \
        "${generator_args[@]}" \
        -DCMAKE_BUILD_TYPE="$flavor" \
        -DGWT_BUILD_FFI_HARNESS=OFF \
        -DGWT_RUST_TARGET="$rust_target" \
        -DGWT_OUTPUT_ARCH=x86_64 \
        "${cmake_source_args[@]}"
    cmake --build "$build_dir" --config "$flavor" --parallel
done
