#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cp -a "$ROOT/workloads/challenge-sha256-v1/implementations/rust-challenge-simd/." "$tmp/"
if [[ -n "${RUSTC_WRAPPER:-}" ]]; then
  test -x "$RUSTC_WRAPPER" || { echo "RUSTC_WRAPPER is not executable" >&2; exit 70; }
elif [[ -f "$HOME/.cargo/config.toml" ]] && rg -q '^rustc-wrapper\s*=.*sccache' "$HOME/.cargo/config.toml"; then
  :
else
  echo "no canonical sccache rustc-wrapper configured; refusing uncached build" >&2
  exit 70
fi
export CARGO_TARGET_DIR="$tmp/target"
cd "$tmp"
cargo +nightly-2026-11-03 run --locked --release --quiet
