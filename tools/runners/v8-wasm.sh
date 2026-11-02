#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
node="${MUSEUM_NODE:-node}"
command -v "$node" >/dev/null || { echo "Node not found: $node" >&2; exit 69; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
uv run --no-project --python 3 "$ROOT/workloads/hashwx-fixed-v1/implementations/wasm-fixed/emit_wasm.py" --output "$tmp/hashwx-fixed.wasm" >/dev/null
"$node" "$ROOT/workloads/hashwx-fixed-v1/implementations/wasm-fixed/v8.mjs" "$tmp/hashwx-fixed.wasm"
