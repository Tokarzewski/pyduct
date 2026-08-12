#!/usr/bin/env bash
# Stage the built venti artifacts (venti.wasm + native cdylib) into bin/.
# Usage: ./stage_artifacts.sh [crate-root]
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CRATE="${1:-$(cd "$HERE/../../.." && pwd)}"   # default: repo root (venti/)
BIN="$HERE/bin"
mkdir -p "$BIN"

WASM="$CRATE/target/wasm32-wasip1/release/venti.wasm"
SO="$CRATE/target/release/libventi.so"
DLL="$CRATE/target/release/venti.dll"

copy() { [ -f "$1" ] && cp "$1" "$BIN/" && echo "[venti] staged $(basename "$1")" || echo "[venti] (skip) missing $1"; }

copy "$WASM"
copy "$SO"
copy "$DLL"
echo "[venti] artifacts in $BIN"