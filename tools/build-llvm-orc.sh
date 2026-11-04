#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ $# -ne 1 ]];then echo "usage: $0 OUTPUT_EXECUTABLE" >&2; exit 64;fi
config="${MUSEUM_LLVM_CONFIG:-llvm-config}"
command -v "$config" >/dev/null || { echo "LLVM 21.1.8 development toolchain unavailable: $config" >&2; exit 69; }
version="$("$config" --version)"
[[ "$version" == 21.1.8 ]] || { echo "LLVM 21.1.8 required; found $version" >&2; exit 69; }
# The pinned LLVM21 toolchain contains LLVM-bitcode static archive members.
# Use its Clang compiler and Acer's already installed LLD23 linker, ALWAYS
# through the existing managed sccache client, never an uncached invocation.
client=/usr/local/libexec/sccache-client
compiler="$(dirname "$(readlink -f "$config")")/clang++"
linker="$(dirname "$(readlink -f "$config")")/ld.lld"
[[ -x "$client" && -x "$compiler" && -x "$linker" ]] || {
  echo "managed sccache, pinned Clang21 or LLD unavailable" >&2; exit 70;
}
export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
# The installed LLVM21 linker expects libxml2.so.2, which is not present
# on newer Acer userspace. Use the exact Debian13 library from the target,
# inside this project's build prefix, not a global SONAME compatibility link.
extra_ldflags=()
if [[ -n "${MUSEUM_LLVM_LINK_LIBDIR:-}" ]];then
  [[ -d "$MUSEUM_LLVM_LINK_LIBDIR" ]] || { echo 'LLVM link library directory unavailable' >&2; exit 69; }
  export LD_LIBRARY_PATH="$MUSEUM_LLVM_LINK_LIBDIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  extra_ldflags=("-L$MUSEUM_LLVM_LINK_LIBDIR")
fi
read -r -a cflags <<<"$("$config" --cxxflags)"
read -r -a ldflags <<<"$("$config" --ldflags)"
read -r -a libs <<<"$("$config" --link-static --libs orcjit native core support)"
read -r -a system_libs <<<"$("$config" --system-libs)"
# The prebuilt llvm-config hardcodes the builder's static zstd archive location.
# Link the supported zstd shared ABI instead; both benchmark hosts provide it.
for i in "${!system_libs[@]}"; do
  if [[ "${system_libs[i]}" == /usr/lib/*/libzstd.a ]]; then system_libs[i]=-lzstd; fi
done
"$client" "$compiler" "${cflags[@]}" -O2 -std=c++20 -fuse-ld="$linker" -DNDEBUG -Wall -Wextra -Wpedantic \
  "$ROOT/workloads/hashwx-fixed-v1/implementations/llvm-orc/main.cpp" \
  -o "$1" "${extra_ldflags[@]}" "${ldflags[@]}" -Wl,--as-needed "${libs[@]}" "${system_libs[@]}" -pthread -ldl -lz
