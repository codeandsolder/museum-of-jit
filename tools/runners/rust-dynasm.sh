#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cp -a "$ROOT/workloads/hashwx-fixed-v1/implementations/rust-dynasm/." "$tmp/"
# Never guess a host's sccache transport. Acer/Asus carry their canonical
# rustc-wrapper in Cargo config; benchmark-only hosts must export RUSTC_WRAPPER
# plus their canonical client endpoint before invoking this runner.
if [[ -n "${RUSTC_WRAPPER:-}" ]]; then
  test -x "$RUSTC_WRAPPER" || { echo "RUSTC_WRAPPER is not executable: $RUSTC_WRAPPER" >&2; exit 70; }
elif [[ -f "$HOME/.cargo/config.toml" ]] && rg -q '^rustc-wrapper\s*=.*sccache' "$HOME/.cargo/config.toml"; then
  : # Cargo owns the wrapper configuration.
else
  echo "no canonical sccache rustc-wrapper configured; refusing an uncached Rust build" >&2
  exit 70
fi
export CARGO_TARGET_DIR="$tmp/target"
cd "$tmp"
cargo +1.99.0 run --locked --release --quiet
