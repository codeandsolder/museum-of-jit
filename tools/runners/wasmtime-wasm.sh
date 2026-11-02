#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
py="${MUSEUM_WASMTIME_PY:-python3}"
command -v "$py" >/dev/null || { echo "Wasmtime Python not found: $py" >&2; exit 69; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
uv run --no-project --python 3 "$ROOT/workloads/hashwx-fixed-v1/implementations/wasm-fixed/emit_wasm.py" --output "$tmp/hashwx-fixed.wasm" >/dev/null
"$py" "$ROOT/workloads/hashwx-fixed-v1/implementations/wasm-fixed/wasmtime_host.py" "$tmp/hashwx-fixed.wasm"
