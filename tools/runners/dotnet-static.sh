#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
dotnet="${MUSEUM_DOTNET:-dotnet}"; command -v "$dotnet" >/dev/null || { echo "dotnet not found" >&2; exit 69; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cp -a "$ROOT/workloads/hashwx-fixed-v1/implementations/dotnet-static/." "$tmp/"
export DOTNET_CLI_HOME="$tmp/home" DOTNET_NOLOGO=1 DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1
"$dotnet" run --project "$tmp/HashwxCs.csproj" -c Release --nologo
