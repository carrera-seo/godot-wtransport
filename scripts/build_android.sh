#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
ndk="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
godot_cpp="${GWT_GODOT_CPP_SOURCE:-}"
api="${GWT_ANDROID_API:-24}"

if [[ -z "$ndk" || -z "$godot_cpp" ]]; then
    echo "ANDROID_NDK_HOME and GWT_GODOT_CPP_SOURCE are required" >&2
    exit 2
fi

case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) host="darwin-x86_64" ;;
    Darwin-x86_64) host="darwin-x86_64" ;;
    Linux-x86_64) host="linux-x86_64" ;;
    *) echo "Unsupported NDK host" >&2; exit 2 ;;
esac

linker="$ndk/toolchains/llvm/prebuilt/$host/bin/aarch64-linux-android${api}-clang"
archiver="$ndk/toolchains/llvm/prebuilt/$host/bin/llvm-ar"
if [[ ! -x "$linker" ]]; then
    echo "Android NDK linker not found: $linker" >&2
    exit 2
fi

rustup target add aarch64-linux-android
generator_args=(-G "Unix Makefiles")
if command -v ninja >/dev/null 2>&1; then
    generator_args=(-G Ninja)
fi
for flavor in Debug Release; do
    lower="$(printf '%s' "$flavor" | tr '[:upper:]' '[:lower:]')"
    build_dir="$root/build-android-$lower-arm64"
    cmake -S "$root" -B "$build_dir" "${generator_args[@]}" \
        -DCMAKE_BUILD_TYPE="$flavor" \
        -DCMAKE_TOOLCHAIN_FILE="$ndk/build/cmake/android.toolchain.cmake" \
        -DANDROID_ABI=arm64-v8a \
        -DANDROID_PLATFORM="android-$api" \
        -DGWT_BUILD_FFI_HARNESS=OFF \
        -DGWT_GODOT_CPP_SOURCE="$godot_cpp" \
        -DGWT_RUST_TARGET=aarch64-linux-android \
        -DGWT_RUST_LINKER_ENV="CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$linker;CC_aarch64_linux_android=$linker;AR_aarch64_linux_android=$archiver" \
        -DGWT_OUTPUT_ARCH=arm64
    cmake --build "$build_dir" --parallel
done
