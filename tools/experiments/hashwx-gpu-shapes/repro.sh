#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    printf 'Usage: %s BUILD_DIRECTORY\n' "$0" >&2
    exit 2
fi

source_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
build_dir="$(realpath -m -- "$1")"
mkdir -p -- "$build_dir"

nvcc=/usr/local/cuda/bin/nvcc
if [ ! -x "$nvcc" ]; then
    printf 'CUDA nvcc required at %s\n' "$nvcc" >&2
    exit 2
fi

compile() {
    label="$1"
    shared="$2"
    branchless="$3"
    "$nvcc" -O3 -std=c++17 -arch=sm_86 -Xptxas=-v \
        -DGROUP_SIZE=1 -DPITCH_WORDS=2049 \
        -DUSE_SHARED="$shared" -DHASHWX_BRANCHLESS_LOAD="$branchless" \
        -I"$source_dir/fixtures" "$source_dir/bench/shape.cu" \
        -o "$build_dir/$label"
}

compile global-original 0 0
compile global-branchless 0 1
compile shared-original 1 0
compile shared-branchless 1 1

(
    cd "$source_dir/fixtures"
    for count in 4096 16384 65536; do
        "$build_dir/global-original" "$count" 32 0 9
        "$build_dir/global-branchless" "$count" 4 0 9
        "$build_dir/shared-original" "$count" 2 0 9
        "$build_dir/shared-branchless" "$count" 2 0 9
    done
)
