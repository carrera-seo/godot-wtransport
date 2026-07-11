#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: smoke_ffi_integration.sh <harness-executable> <server-executable>" >&2
    exit 2
fi

work="$(mktemp -d)"
server_log="$work/server.log"
port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
server_pid=""

cleanup() {
    if [[ -n "$server_pid" ]]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$work"
}
trap cleanup EXIT

"$2" --listen "127.0.0.1:$port" --san 127.0.0.1 >"$server_log" 2>&1 &
server_pid=$!
for _attempt in {1..100}; do
    if grep -q '"event":"ready"' "$server_log"; then break; fi
    if ! kill -0 "$server_pid" 2>/dev/null; then
        cat "$server_log" >&2
        exit 1
    fi
    sleep 0.1
done
grep -q '"event":"ready"' "$server_log"
hash="$(python3 -c 'import json,sys; print(json.loads(open(sys.argv[1]).readline())["certificate_hash"])' "$server_log")"
"$1" "https://127.0.0.1:$port/echo" "$hash"
