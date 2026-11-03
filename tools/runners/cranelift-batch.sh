#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export MUSEUM_CRANELIFT_BATCH=1
exec "$ROOT/tools/runners/cranelift-direct.sh"
