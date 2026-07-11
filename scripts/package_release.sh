#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="${1:-0.1.0-dev}"
dist="$root/dist"
stage="$dist/stage"

rm -rf "$stage"
mkdir -p "$stage/addons/godot_wtransport" "$dist"
cp -R "$root/demo/addons/godot_wtransport/." "$stage/addons/godot_wtransport/"
find "$stage" -name '.DS_Store' -delete
find "$stage" -exec touch -t 198001010000 {} +

archive="$dist/godot-wtransport-$version.zip"
rm -f "$archive" "$archive.sha256"
(cd "$stage" && python3 -m zipfile -c "$archive" addons)
python3 -c 'import hashlib, pathlib, sys; p=pathlib.Path(sys.argv[1]); print(hashlib.sha256(p.read_bytes()).hexdigest(), "", p.name)' "$archive" > "$archive.sha256"

server="${GWT_DEV_SERVER:-$root/target/release/godot-wtransport-dev-server}"
if [[ -f "$server.exe" ]]; then
    server="$server.exe"
fi
if [[ -f "$server" ]]; then
    cp "$server" "$dist/godot-wtransport-dev-server-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)${server##*godot-wtransport-dev-server}"
fi

python3 -m zipfile -t "$archive"
