#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if command -v nix >/dev/null 2>&1; then
    nix run .#update-demo-wasm
elif command -v wasm-pack >/dev/null 2>&1; then
    wasm-pack build ./applications/wasm --target web --out-dir ../demopage/pkg
else
    echo "Error: nix or wasm-pack is required to build WASM package." >&2
    exit 1
fi
