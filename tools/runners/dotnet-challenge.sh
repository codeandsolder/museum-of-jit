#!/usr/bin/env bash
set -euo pipefail
mode="${1:?need portable|scalar|bcl}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
dotnet="${MUSEUM_DOTNET:-dotnet}"
command -v "$dotnet" >/dev/null || { echo "dotnet not found" >&2; exit 69; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cp -a "$ROOT/workloads/challenge-sha256-v1/implementations/dotnet-portable/." "$tmp/"
export DOTNET_CLI_HOME="$tmp/home" DOTNET_NOLOGO=1 DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1
export DOTNET_CLI_TELEMETRY_OPTOUT=1
"$dotnet" publish "$tmp/ChallengeSha256.csproj" -c Release -o "$tmp/out" --nologo -v quiet >&2
"$dotnet" "$tmp/out/ChallengeSha256.dll" --mode "$mode"
