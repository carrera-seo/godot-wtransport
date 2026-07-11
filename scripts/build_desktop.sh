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
        if [[ -n "${VCToolsInstallDir:-}" ]]; then
            msvc_tools="$(cygpath -u "$VCToolsInstallDir")"
            msvc_linker="$msvc_tools/bin/Hostx64/x64/link.exe"
            if [[ ! -x "$msvc_linker" ]]; then
                echo "MSVC linker not found: $msvc_linker" >&2
                exit 2
            fi
            export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="$msvc_linker"
        fi
        ;;
    *)
        echo "Unsupported desktop platform" >&2
        exit 2
        ;;
esac

if command -v rustup >/dev/null 2>&1; then
    rustup target add "$rust_target"
elif ! rustc -vV | grep -q "host: $rust_target"; then
    echo "rustup is required to install the non-host target $rust_target" >&2
    exit 2
fi
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
