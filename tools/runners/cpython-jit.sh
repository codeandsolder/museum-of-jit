#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
py="${MUSEUM_CPYTHON:-python3}"; command -v "$py" >/dev/null || { echo "CPython not found" >&2; exit 69; }
PYTHON_JIT=1 "$py" "$ROOT/workloads/hashwx-fixed-v1/implementations/python-generated/bench.py" --engine cpython
