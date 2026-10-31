# Curated results

Tracked result files contain only accepted benchmark runs. Failed bring-up attempts, smoke tests, and invalid measurements stay out of the curated dataset.

## EPYC 9655 / Zen 5 — 2026-10-31

`epyc-9655-zen5-2026-10-31.json` is the first Museum dataset from cold-storage (`AMD EPYC 9655 96-Core Processor`, Debian 13). All entries use `hashwx-fixed-v1` corpus SHA-256 `6fbc50324ca77396e097be37705e68bc24e6b3267c308e7cd5f50568195be94b`.

| implementation | median H/s | relative to Rust/dynasm |
|---|---:|---:|
| `rust-dynasm` | 442,368 | 100.0% |
| `dotnet-dynamic-pointer` | 412,055 | 93.1% |
| `dotnet-static` | 290,235 | 65.6% |

The accepted Rust run used the host's canonical TCP/4226 sccache client with `SCCACHE_START_SERVER=0`. An earlier run that emitted a local-compilation fallback warning was discarded.

The recovered historical `.NET dynamic-il` adapter is not in this success file because .NET SDK/runtime 10.0.403 rejects it with `InvalidProgramException`. PyPy 8.0.1 generated-source + `__pypy__.intop` passes correctness but only reached ~1.6 kH/s in the early 5k warm/sample probe; it remains a bring-up result until we decide on a stable warmup policy for pathological tracing cases.

GraalVM 25.0.4 straight-line Java did not complete the standard 30k-warmup/7×300k run within the initial 120-second harness budget. That is a useful compiler-behavior finding, but it is not represented as a throughput result until the result schema grows explicit failure/timeout records.
