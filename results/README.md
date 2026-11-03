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

GraalVM 25.0.4 straight-line Java did not complete the standard 30k-warmup/7×300k run within the initial 120-second harness budget. Later diagnosis showed that this is a monolithic-method JIT pathology rather than source compilation failure; the split-helper shape is measured separately in the 2026-11-02 dataset below.

## EPYC 9655 / Zen 5 — 2026-11-02

`epyc-9655-zen5-2026-11-02.json` adds accepted single-logical-CPU (`cpu_affinity = [0]`) measurements for the recovered Java split shape and for a shared fixed-program WebAssembly module. Every entry uses 30,000 warmup hashes, seven measured samples of 300,000 hashes, and the canonical `fd5d450b767035c5` correctness probe.

| implementation / runtime | median H/s | relative to Rust/dynasm | sample range H/s |
|---|---:|---:|---:|
| `v8-wasm` — Node 26.12.1 / V8 14.6 | 399,602 | 90.3% | 389,488–411,860 |
| `wasmtime-wasm` — Wasmtime 50.0.1 / Cranelift | 348,988 | 78.9% | 338,858–367,275 |
| `java-split` — GraalVM 25.0.4 | 291,467 | 65.9% | 277,290–294,358 |
| `java-split` — OpenJDK 21.0.13 | 244,819 | 55.3% | 196,772–250,221 |

The V8 and Wasmtime rows execute the exact same 14,166-byte generated WebAssembly module. Its measured loop runs inside Wasm, so the throughput figures do not include one host-language call per hash. Before promotion, the module's exported `hash()` matched the released `hashwx-rs` 0.1.0 Wasm emitter for 100,000 deterministic inputs. V8 compile+instantiate latency was 300.444 µs; Wasmtime was 9,581.362 µs with parallel compilation explicitly disabled.

The Java runs preserve all samples because the first measured sample is visibly colder, especially on OpenJDK 21. The previously tested `@ForceInline` split variant was slightly worse than the ordinary Graal split in the diagnostic run and is not promoted as a separate canonical result.

## EPYC 9655 / Zen 5 — 2026-11-03

`epyc-9655-zen5-2026-11-03.json` adds the direct RPython/PyPy tracing meta-JIT exhibit, pinned to one logical CPU (`cpu_affinity = [0]`). The translated runtime interprets the canonical fixed HashWX bytecode under an explicit `JitDriver`; the bytecode image is generated from `program.json`, promoted as immutable runtime state, and the eight-instruction program body is marked `unroll_safe` so specialization happens at program boundaries rather than at every opcode.

| implementation / runtime | median H/s | relative to Rust/dynasm | sample range H/s |
|---|---:|---:|---:|
| `rpython-jitdriver` — PyPy RPython 8.0.1 | 75,356 | 17.0% | 63,121–78,653 |

The accepted run uses the standard 30,000 warmup hashes and seven samples of 300,000 hashes. Throughput still rises across the measured samples (63.1 → 78.7 kH/s at the high end), so all samples are retained rather than pretending the meta-JIT has a sharp warmup boundary. A same-shape `jit-summary` diagnostic recorded 75 loops and 8,969 bridges: the workload's data-dependent bounded program-repeat control flow remains a poor fit for PyPy's tracing strategy even after moving the merge point to one per program iteration.

A follow-up experiment made the branch counter itself green to turn those exits into bounded specialized states. It was rejected and reverted: the translated binary was correct, but a 30k-warmup/300k-sample diagnostic spent over a minute compiling/warming and never reached the measured sample before termination. The curated row therefore represents the simpler general interpreter shape, not a corpus-specific JIT-state explosion.

## Completed experimental exhibit: CPython 3.15 tier-2 JIT — 2026-11-03

The checksum-verified **CPython 3.15.2** experimental tier-2 JIT build (Clang 21.1.8, LLVM InstrProf PGO, full LTO, audited LLD 23.1.4) passed the 43-file PGO test run, installation, real tier-2 executor smoke, relocation, and target compatibility check (maximum referenced GLIBC 2.38). The build script is preserved in `tools/build-cpython-jit.sh`, with an accepted artifact on Acer and a transferred runtime on cold-storage. The working build is **not** evidence of competitive generated-Python performance.

For the fixed HashWX program on one EPYC 9655 logical CPU, a 30k-warmup/7×10k diagnostic gave **458.485 H/s** median with JIT enabled versus **1,185.772 H/s** with JIT disabled on the **same binary**; outputs matched. With 30k warmup and 7×5k samples, raising the initial side-exit threshold from 4000 to 8191 improved the enabled-JIT median from **533.417 to 615.840 H/s** and reduced newly installed executors from 45 to 9 during samples. Even the tuned diagnostic rate was roughly **700× below** the independently measured direct Rust/dynasm median of **442,368 H/s**, but note the sample lengths were different. This is a clear noncompetitive result, not a like-for-like canonical leaderboard figure.

**Disposition: experiment concluded and not pursued further for HashWX optimization.** Keep the reproducible builder, generated-Python implementation, and raw diagnostics as an instructive negative result. Do **not** add an invented 7×300k CPython row to the accepted dataset. See [`diagnostics/README.md`](diagnostics/README.md) for all measurement shapes, per-sample rates, executor counts, source provenance, and reproduction commands.
