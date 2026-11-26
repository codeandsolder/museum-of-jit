#!/usr/bin/env bash
# Compile-time-only native kernel shell: large instruction capacity, high
# register allocation, preserved entry ABI, and two EXIT sites.
# Generated PTX is never executed: native JIT always replaces its SASS body.
set -euo pipefail
if [[ $# != 2 ]]; then
    echo "usage: build-shell.sh OUTPUT_PTX VOLATILE_STORE_COUNT" >&2
    exit 2
fi
target="$(realpath -m -- "$1")"
stores="$2"
if [[ ! "$stores" =~ ^[1-9][0-9]*$ || "$stores" -gt 32000 ]]; then
    echo "store count must be 1..32000" >&2
    exit 2
fi
mkdir -p -- "$(dirname "$target")"
{
    cat <<'PTX'
.version 8.8
.target sm_86
.address_size 64
.extern .shared .align 8 .b8 scratch[];
.visible .entry hash_batch(
  .param .u64 input, .param .u64 output, .param .u32 count)
.maxntid 2,1,1
{
.reg .pred %p<1>;
.reg .b32 %u<112>;
.reg .b64 %addr, %shaddr, %longvalue;
ld.param.u64 %addr,[input];
ld.param.u32 %u0,[count];
mov.u32 %u1,%tid.x;
setp.ge.u32 %p0,%u1,%u0;
@%p0 bra SKIP;
mov.u64 %shaddr,scratch;
mov.u64 %longvalue,0;
st.volatile.shared.u64 [%shaddr],%longvalue;
PTX
    # Force many registers live simultaneously so the generated shell reserves
    # enough for all observed HashWX variants.
    for ((i=2;i<112;i++)); do
        printf 'ld.volatile.global.u32 %%u%d,[%%addr];\n' "$i"
    done
    for ((i=2;i<112;i++)); do
        printf 'st.volatile.global.u32 [%%addr],%%u%d;\n' "$i"
    done
    for ((i=0;i<stores;i++)); do
        printf 'st.volatile.global.u32 [%%addr],%%u2;\n'
    done
    cat <<'PTX'
ret;
SKIP:
ret;
}
PTX
} > "$target"
printf 'Generated native kernel calibration shell PTX: %s, padding stores=%s\n' "$target" "$stores"
