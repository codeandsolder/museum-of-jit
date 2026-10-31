#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
py="${MUSEUM_PYPY:-pypy3}"; command -v "$py" >/dev/null || { echo "PyPy not found" >&2; exit 69; }
"$py" "$ROOT/workloads/hashwx-fixed-v1/implementations/python-generated/bench.py" --engine pypy
