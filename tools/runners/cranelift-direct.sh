#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export MUSEUM_PROGRAM_JSON="$ROOT/workloads/hashwx-fixed-v1/program.json"
if [[ -n "${MUSEUM_CRANELIFT_BIN:-}" ]]; then
    [[ -x "$MUSEUM_CRANELIFT_BIN" ]] || { echo "Cranelift executable unavailable: $MUSEUM_CRANELIFT_BIN" >&2; exit 69; }
    exec "$MUSEUM_CRANELIFT_BIN"
fi
if [[ -z "${RUSTC_WRAPPER:-}" ]] && \
   ! { [[ -f "$HOME/.cargo/config.toml" ]] && rg -q '^rustc-wrapper\s*=.*sccache' "$HOME/.cargo/config.toml"; }; then
    echo 'No canonical sccache rustc-wrapper configured; refusing uncached Cranelift build' >&2
    exit 70
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cp -a "$ROOT/workloads/hashwx-fixed-v1/implementations/cranelift-direct/." "$tmp/"
export CARGO_TARGET_DIR="$tmp/target"
cargo +1.99.0 build --locked --release --manifest-path "$tmp/Cargo.toml" --quiet
"$CARGO_TARGET_DIR/release/museum-hashwx-cranelift-direct"
