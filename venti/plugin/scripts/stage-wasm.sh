#!/usr/bin/env bash
# Stage the venti WASM core beside the plugin DLL (issue #17).
set -euo pipefail
cd "$(dirname "$0")/../.."            # to crate root
SRC="${1:-target/wasm32-wasip1/release/venti.wasm}"
if [ ! -f "$SRC" ]; then
  echo "[venti] venti.wasm not found at $SRC; run scripts/build-wasm.sh --release first" >&2
  exit 1
fi
mkdir -p plugin/bin/Release
cp "$SRC" plugin/bin/Release/venti.wasm
echo "[venti] staged venti.wasm into plugin/bin/Release"
