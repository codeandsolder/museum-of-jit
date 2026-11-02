#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
javac="${MUSEUM_JAVAC:-javac}"; java="${MUSEUM_JAVA:-java}"; command -v "$javac" >/dev/null || { echo "javac not found" >&2; exit 69; }; command -v "$java" >/dev/null || exit 69
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
javac_args=()
if [[ -n "${MUSEUM_JAVAC_RELEASE:-}" ]]; then javac_args+=(--release "$MUSEUM_JAVAC_RELEASE"); fi
"$javac" "${javac_args[@]}" -d "$tmp" "$ROOT/workloads/hashwx-fixed-v1/implementations/java-split/HashWxSplit.java"
"$java" -cp "$tmp" HashWxSplit
