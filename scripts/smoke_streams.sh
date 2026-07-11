#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: smoke_streams.sh <godot-executable> <server-executable>" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
project="$(mktemp -d)"
certificate="$project/server.pem"
server_log="$project/server.log"
port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
server_pid=""

cleanup() {
    if [[ -n "$server_pid" ]]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$project"
}
trap cleanup EXIT

cp -R "$root/demo/." "$project/"
mkdir -p "$project/.godot"
printf '%s\n' 'res://addons/godot_wtransport/godot-wtransport.gdextension' > \
    "$project/.godot/extension_list.cfg"

"$2" --listen "127.0.0.1:$port" --san 127.0.0.1 \
    --write-generated-cert "$certificate" >"$server_log" 2>&1 &
server_pid=$!
for _attempt in {1..100}; do
    if grep -q '"event":"ready"' "$server_log"; then
        break
    fi
    if ! kill -0 "$server_pid" 2>/dev/null; then
        cat "$server_log" >&2
        exit 1
    fi
    sleep 0.1
done
grep -q '"event":"ready"' "$server_log"

GWT_TEST_URL="https://127.0.0.1:$port/echo" \
GWT_TEST_CUSTOM_CA="$certificate" \
    "$1" --headless --path "$project" \
    --log-file "$project/godot-streams.log" \
    --script res://tests/streams.gd
