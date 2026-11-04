#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export MUSEUM_PROGRAM_JSON="$ROOT/workloads/hashwx-fixed-v1/program.json"
if [[ -n "${MUSEUM_LLVM_ORC_LIBDIR:-}" ]];then
  [[ -d "$MUSEUM_LLVM_ORC_LIBDIR" ]] || { echo 'LLVM ORC library directory unavailable' >&2; exit 69; }
  export LD_LIBRARY_PATH="$MUSEUM_LLVM_ORC_LIBDIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
if [[ -n "${MUSEUM_LLVM_ORC_BIN:-}" ]];then
  [[ -x "$MUSEUM_LLVM_ORC_BIN" ]] || { echo "LLVM ORC executable unavailable: $MUSEUM_LLVM_ORC_BIN" >&2; exit 69; }
  exec "$MUSEUM_LLVM_ORC_BIN"
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
"$ROOT/tools/build-llvm-orc.sh" "$tmp/llvm-orc-hashwx"
"$tmp/llvm-orc-hashwx"
