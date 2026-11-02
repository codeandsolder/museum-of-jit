#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
: "${MUSEUM_RPYTHON_HASHWX:?MUSEUM_RPYTHON_HASHWX must point to the translated RPython executable}"
test -x "$MUSEUM_RPYTHON_HASHWX" || { echo "not executable: $MUSEUM_RPYTHON_HASHWX" >&2; exit 69; }
uv run --no-project --python 3 "$ROOT/workloads/hashwx-fixed-v1/implementations/rpython-jitdriver/host.py"
