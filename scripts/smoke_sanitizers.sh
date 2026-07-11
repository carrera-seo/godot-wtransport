#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "Usage: smoke_sanitizers.sh" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
binary="$(mktemp)"
trap 'rm -f "$binary"' EXIT
library="$root/target/x86_64-unknown-linux-gnu/debug/libgodot_wtransport_ffi.a"

cc -fsanitize=address,undefined -fno-omit-frame-pointer \
    -I "$root/rust/ffi/include" \
    "$root/tests/ffi/lifecycle.c" "$library" \
    -lpthread -ldl -lm -lrt -lutil -o "$binary"

ASAN_OPTIONS="detect_leaks=1:halt_on_error=1:abort_on_error=1" \
UBSAN_OPTIONS="halt_on_error=1:print_stacktrace=1" \
    "$binary"
