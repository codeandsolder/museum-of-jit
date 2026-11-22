#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [[ -n "${MUSEUM_CHALLENGE_RUST_BIN:-}" ]]; then
  test -x "$MUSEUM_CHALLENGE_RUST_BIN" || { echo "missing prebuilt Rust binary" >&2; exit 70; }
  printf 'BINARY_SHA256 %s\n' "$(sha256sum "$MUSEUM_CHALLENGE_RUST_BIN" | cut -d' ' -f1)"
  exec "$MUSEUM_CHALLENGE_RUST_BIN"
fi

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
if [[ -n "${RUSTC_WRAPPER:-}" ]]; then
  test -x "$RUSTC_WRAPPER" || { echo "RUSTC_WRAPPER is not executable" >&2; exit 70; }
elif [[ -f "$HOME/.cargo/config.toml" ]] && rg -q '^rustc-wrapper\s*=.*sccache' "$HOME/.cargo/config.toml"; then
  :
else
  echo "no canonical sccache rustc-wrapper configured; refusing uncached build" >&2
  exit 70
fi
build=(cargo +nightly-2026-11-03 build --locked --release --quiet
  --manifest-path "$ROOT/workloads/challenge-sha256-v1/implementations/rust-challenge-simd/Cargo.toml"
  --target-dir "$tmp/target" --message-format=json-render-diagnostics)
if [[ -n "${MUSEUM_BUILD_CPUSET:-}" ]]; then
  build=(taskset -c "$MUSEUM_BUILD_CPUSET" "${build[@]}")
fi
binary=$("${build[@]}" | jq -ers '
  [.[] | select(.reason == "compiler-artifact" and
                 .target.name == "museum-challenge-sha256-reference" and
                 .executable != null) | .executable] |
  if length == 1 then .[0] else error("expected one Cargo benchmark executable") end')
test -x "$binary" || { echo "Cargo benchmark executable unavailable" >&2; exit 70; }
printf 'BINARY_SHA256 %s\n' "$(sha256sum "$binary" | cut -d' ' -f1)"
"$binary"
