#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: smoke_web_export.sh <godot-executable>" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
output="$root/dist/web"
mkdir -p "$output"
"$1" --headless --path "$root/demo" \
    --log-file "$output/godot-web-export.log" \
    --export-release Web "$output/index.html"
test -s "$output/index.html"
test -s "$output/index.js"
test -s "$output/index.wasm"
grep -q "Godot" "$output/index.html"
if grep -qE "SCRIPT ERROR|Parse Error|Compile Error" "$output/godot-web-export.log"; then
    echo "Web export log contains script compilation errors" >&2
    exit 1
fi
echo "godot-wtransport Web export smoke passed"
