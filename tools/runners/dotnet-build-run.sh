#!/usr/bin/env bash
set -euo pipefail
project="${1:?need project path}"
shift
dotnet="${MUSEUM_DOTNET:-dotnet}"
command -v "$dotnet" >/dev/null || { echo "dotnet not found" >&2; exit 69; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
export TMPDIR="$tmp"
export DOTNET_CLI_HOME="$tmp/home" DOTNET_NOLOGO=1 DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1
export DOTNET_CLI_TELEMETRY_OPTOUT=1
build=("$dotnet" publish "$project" --artifacts-path "$tmp/artifacts" --disable-build-servers -c Release -o "$tmp/out" --nologo -v quiet)
if [[ -n "${MUSEUM_BUILD_CPUSET:-}" ]]; then
  build=(taskset -c "$MUSEUM_BUILD_CPUSET" "${build[@]}")
fi
"${build[@]}" >&2
assembly="$(basename "$project" .csproj).dll"
"$dotnet" "$tmp/out/$assembly" "$@"
