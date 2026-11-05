#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [[ -n "${MUSEUM_CHALLENGE_RUST_BIN:-}" ]]; then
  test -x "$MUSEUM_CHALLENGE_RUST_BIN" || { echo "missing prebuilt Rust binary" >&2; exit 70; }
  printf 'BINARY_SHA256 %s\n' "$(sha256sum "$MUSEUM_CHALLENGE_RUST_BIN" | cut -d' ' -f1)"
  exec "$MUSEUM_CHALLENGE_RUST_BIN"
fi

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
if [[ -n "${MUSEUM_BUILD_CPUSET:-}" ]]; then
  taskset -c "$MUSEUM_BUILD_CPUSET" cargo +nightly-2026-11-03 build --locked --release --quiet
else
  cargo +nightly-2026-11-03 build --locked --release --quiet
fi
printf 'BINARY_SHA256 %s\n' "$(sha256sum "$CARGO_TARGET_DIR/release/museum-challenge-sha256-reference" | cut -d' ' -f1)"
exec "$CARGO_TARGET_DIR/release/museum-challenge-sha256-reference"
