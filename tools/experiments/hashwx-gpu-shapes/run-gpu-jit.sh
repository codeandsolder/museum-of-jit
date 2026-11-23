#!/usr/bin/env bash
# Full HashWX PTX-to-SASS JIT, automatic GPU architecture bootstrap, and oracles.
# All generated binaries, native cubins, and calibration stay under BUILD_DIR.
set -euo pipefail

if [[ $# -lt 2 || $# -gt 3 ]]; then
    echo "usage: run-gpu-jit.sh GENERATED_SEEDS_DIRECTORY BUILD_DIRECTORY [SAMPLES]" >&2
    exit 2
fi

source_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
seeds_dir="$(realpath -- "$1")"
build_dir="$(realpath -m -- "$2")"
samples=7
if [[ $# == 3 ]]; then samples="$3"; fi
if [[ ! "$samples" =~ ^[1-9][0-9]*$ ]]; then
    echo "SAMPLES must be a positive integer" >&2
    exit 2
fi
mkdir -p -- "$build_dir"

ptxas=/usr/local/cuda/bin/ptxas
nvcc=/usr/local/cuda/bin/nvcc
if [[ ! -x "$ptxas" || ! -x "$nvcc" ]]; then
    echo "CUDA toolkit ptxas and nvcc are required" >&2
    exit 2
fi
if [[ ! -f "$seeds_dir/seed-000/kernel.ptx" ]]; then
    echo "Missing seed-000/kernel.ptx in $seeds_dir" >&2
    exit 2
fi

cargo build --offline --release \
    --manifest-path "$source_dir/bootstrap/Cargo.toml" \
    --target-dir "$build_dir/bootstrap-target"
bootstrap="$build_dir/bootstrap-target/release/hashwx-sass-bootstrap"
"$bootstrap" auto "$seeds_dir/seed-000/kernel.ptx" "$build_dir/architecture"
sm="$(sed -n 's/^architecture=//p' "$build_dir/architecture/manifest.txt" | head -1)"
if [[ ! "$sm" =~ ^sm_[0-9]{2,3}$ ]]; then
    echo "Invalid calibrated GPU architecture: $sm" >&2
    exit 1
fi

# One SM-generation native shell, generated only when its input/compiler changes.
mkdir -p "$build_dir/native-shell"
if [[ ! -f "$build_dir/native-shell/shell.ptx" || "$source_dir/bootstrap/build-shell.sh" -nt "$build_dir/native-shell/shell.ptx" ]]; then
    bash "$source_dir/bootstrap/build-shell.sh" "$build_dir/native-shell/shell.ptx" 18000
fi
"$bootstrap" auto "$build_dir/native-shell/shell.ptx" "$build_dir/native-shell/calibration"
shell="$build_dir/native-shell/calibration/reference.cubin"
patched_dir="$build_dir/native-patched"
mkdir -p "$patched_dir"

native_dir="$build_dir/compiled"
mkdir -p "$native_dir"
toolchain="$("$ptxas" --version | tail -1)"
shell_digest="$(sha256sum "$shell" | cut -d" " -f1)"
for seed in "$seeds_dir"/seed-*; do
    [[ -d "$seed" ]] || continue
    if [[ ! -f "$seed/kernel.ptx" || ! -f "$seed/inputs.bin" ||
          ! -f "$seed/expected.bin" ]]; then
        echo "Incomplete seed corpus: $seed" >&2
        exit 1
    fi
    name="$(basename "$seed")"
    out="$native_dir/$name"
    mkdir -p "$out"
    digest="$(sha256sum "$seed/kernel.ptx" | cut -d' ' -f1)"
    stamp="$(printf '%s\n%s\n%s\n' "$digest" "$sm" "$toolchain")"
    if [[ -f "$out/stamp.txt" && -f "$out/kernel.cubin" &&
          "$(cat "$out/stamp.txt")" == "$stamp" ]]; then
        echo "jit_cache_hit=true seed=$name arch=$sm"
    else
        sed -E "s/^\.target sm_[0-9]+/.target $sm/" \
            "$seed/kernel.ptx" > "$out/kernel.ptx"
        echo "jit_cache_hit=false seed=$name arch=$sm"
        "$ptxas" -O3 "-arch=$sm" "$out/kernel.ptx" -o "$out/kernel.cubin"
        printf '%s' "$stamp" > "$out/stamp.txt"
    fi
    ln -sfn "$(realpath -- "$seed/inputs.bin")" "$out/inputs.bin"
    ln -sfn "$(realpath -- "$seed/expected.bin")" "$out/expected.bin"
    # Transplant the exact native instruction sequence into the once-per-SM
    # oversized shell, updating the exit-offset metadata and checking ABI.
    patched="$patched_dir/$name"
    mkdir -p "$patched"
    donor_digest="$(sha256sum "$out/kernel.cubin" | cut -d' ' -f1)"
    patch_stamp="$(printf '%s\n%s\n' "$shell_digest" "$donor_digest")"
    if [[ ! -f "$patched/stamp.txt" || ! -f "$patched/kernel.cubin" ||
          "$(cat "$patched/stamp.txt")" != "$patch_stamp" ]]; then
        "$bootstrap" patch "$shell" "$out/kernel.cubin" "$patched/kernel.cubin"
        printf '%s' "$patch_stamp" > "$patched/stamp.txt"
    else
        echo "native_patch_cache_hit=true seed=$name arch=$sm"
    fi
    ln -sfn "$(realpath -- "$seed/inputs.bin")" "$patched/inputs.bin"
    ln -sfn "$(realpath -- "$seed/expected.bin")" "$patched/expected.bin"
done
if [[ ! -x "$build_dir/load-cubin" ||
      "$source_dir/bench/load-cubin.cu" -nt "$build_dir/load-cubin" ]]; then
    "$nvcc" -O2 -std=c++17 "-arch=$sm" \
        "$source_dir/bench/load-cubin.cu" \
        -o "$build_dir/load-cubin" -lcudart -lcuda
fi
"$build_dir/load-cubin" "$patched_dir" "$samples" | tee "$build_dir/verified-results.log"
