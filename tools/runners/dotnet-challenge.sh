#!/usr/bin/env bash
set -euo pipefail
mode="${1:?need portable|optimized|match|match-optimized|scalar|bcl}"
shift
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec "$ROOT/tools/runners/dotnet-build-run.sh" "$ROOT/workloads/challenge-sha256-v1/implementations/dotnet-portable/ChallengeSha256.csproj" --mode "$mode" "$@"
