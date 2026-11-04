#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ $# -ne 1 ]];then echo "usage: $0 OUTPUT_EXECUTABLE" >&2; exit 64;fi
command -v llvm-config >/dev/null || { echo 'LLVM development installation unavailable' >&2; exit 69; }
version="$(llvm-config --version)"
[[ "$version" == 23.1.4 ]] || { echo "LLVM 23.1.4 required; found $version" >&2; exit 69; }
# The canonical strict wrapper delegates to the pre-existing sccache service.
wrapper=/usr/local/libexec/sccache-strict-wrappers/c++
[[ -x "$wrapper" ]] || { echo "managed sccache C++ wrapper unavailable: $wrapper" >&2; exit 70; }
export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
read -r -a cflags <<<"$(llvm-config --cxxflags)"
read -r -a ldflags <<<"$(llvm-config --ldflags)"
read -r -a libs <<<"$(llvm-config --libs orcjit native core support)"
"$wrapper" "${cflags[@]}" -O2 -std=c++20 -DNDEBUG -Wall -Wextra -Wpedantic \
  "$ROOT/workloads/hashwx-fixed-v1/implementations/llvm-orc/main.cpp" \
  -o "$1" "${ldflags[@]}" "${libs[@]}" -pthread -ldl -lz
