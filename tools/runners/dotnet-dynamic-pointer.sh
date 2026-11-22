#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec "$ROOT/tools/runners/dotnet-build-run.sh" "$ROOT/workloads/hashwx-fixed-v1/implementations/dotnet-dynamic-pointer/HashwxDynPointer.csproj" "$@"
