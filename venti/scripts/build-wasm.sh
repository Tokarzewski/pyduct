#!/usr/bin/env bash
# Build `venti` as a self-contained WebAssembly core (cdylib) exposing the
# C-ABI `venti_*` functions for embedding from C#, Python, C++, Node, etc.
#
# Prereqs:
#   rustup target add wasm32-wasip1
#
# Usage:
#   ./scripts/build-wasm.sh            # debug -> target/wasm32-wasip1/debug/venti.wasm
#   ./scripts/build-wasm.sh --release  # release -> target/wasm32-wasip1/release/venti.wasm
set -euo pipefail
cd "$(dirname "$0")/.."

mode="${1:-}"
extra=()
if [[ "$mode" == "--release" ]]; then
  extra+=(--release)
fi

cargo build --target wasm32-wasip1 --lib --no-default-features "${extra[@]}"

artifact="target/wasm32-wasip1/${mode#--}/venti.wasm"
echo "WASM core built: $artifact"
ls -la "$artifact"
