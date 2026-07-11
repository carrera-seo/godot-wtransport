#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: package_desktop_artifacts.sh <platform> <version>" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
platform="$1"
version="$2"
bin="$root/demo/addons/godot_wtransport/bin"
symbols="$root/dist/symbols-$platform"
mkdir -p "$symbols"

case "$platform" in
    macos)
        for library in "$bin"/*.dylib; do
            dsymutil "$library" -o "$symbols/$(basename "$library").dSYM"
        done
        server="$root/target/aarch64-apple-darwin/release/godot-wtransport-dev-server"
        ;;
    linux)
        for library in "$bin"/*.so; do
            objcopy --only-keep-debug "$library" "$symbols/$(basename "$library").debug"
            strip --strip-unneeded "$library"
        done
        server="$root/target/x86_64-unknown-linux-gnu/release/godot-wtransport-dev-server"
        ;;
    windows)
        find "$root" -iname '*.pdb' -exec cp {} "$symbols/" \;
        server="$root/target/x86_64-pc-windows-msvc/release/godot-wtransport-dev-server.exe"
        ;;
    *)
        echo "Unsupported platform: $platform" >&2
        exit 2
        ;;
esac

GWT_DEV_SERVER="$server" "$root/scripts/package_release.sh" "$version"
(cd "$root/dist" && zip -X -r "godot-wtransport-$version-$platform-symbols.zip" "symbols-$platform")
shasum -a 256 "$root/dist/godot-wtransport-$version-$platform-symbols.zip" > \
    "$root/dist/godot-wtransport-$version-$platform-symbols.zip.sha256"
