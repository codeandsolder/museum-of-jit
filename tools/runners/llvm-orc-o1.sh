#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export MUSEUM_LLVM_IR_OPT=O1
exec "$ROOT/tools/runners/llvm-orc.sh"
