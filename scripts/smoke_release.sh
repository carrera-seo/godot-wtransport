#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: smoke_release.sh <godot-executable> <platform>" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
godot="$1"
platform="$2"
case "$platform" in
    macos)
        release="libgodot_wtransport.macos.release.universal.dylib"
        debug="libgodot_wtransport.macos.debug.universal.dylib"
        ;;
    linux)
        release="libgodot_wtransport.linux.release.x86_64.so"
        debug="libgodot_wtransport.linux.debug.x86_64.so"
        ;;
    windows)
        release="godot_wtransport.windows.release.x86_64.dll"
        debug="godot_wtransport.windows.debug.x86_64.dll"
        ;;
    *)
        echo "Unsupported platform: $platform" >&2
        exit 2
        ;;
esac

project="$(mktemp -d)"
trap 'rm -rf "$project"' EXIT
cp -R "$root/demo/." "$project/"
cp "$project/addons/godot_wtransport/bin/$release" \
    "$project/addons/godot_wtransport/bin/$debug"
mkdir -p "$project/.godot"
printf '%s\n' 'res://addons/godot_wtransport/godot-wtransport.gdextension' > \
    "$project/.godot/extension_list.cfg"
"$godot" --headless --path "$project" \
    --log-file "$project/godot-smoke.log" \
    --script res://tests/lifecycle.gd
