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

## EPYC 9655 / Zen 5 — direct Cranelift native IR (2026-11-03)

`epyc-9655-zen5-cranelift-2026-11-03.json` adds an accepted fixed HashWX native Cranelift JIT exhibit. Version **0.137.1**, with explicitly selected **`opt_level=speed`**, consumes the same checked `program.json` as the other adapters, generates all 32 programs directly to Cranelift IR and finalizes a native `hash(u64) -> u64` function. It is **not a WebAssembly compilation**. The exact compiled function matched all **100,000 deterministic inputs** against the separately generated V8/Wasm oracle; the full output SHA-256 was `f9a9a50baa9201bd847fc39e8f2b88036883de0a298b322dd46197029a987cc3` for both engines.

The accepted EPYC 9655 run was pinned to **CPU0**, with **30,000 warmup hashes** and **seven × 300,000-hash samples**. All sinks equal `1842ddbd4c0fefed`, all samples passed, the executable was SHA256-matched across Acer/cold-storage, and the run recorded source commit `23f8043`.

| implementation / policy | median H/s | sample range H/s | build µs | relation to dynasm 442,368 H/s |
|---|---:|---:|---:|---:|
| `cranelift-direct` 0.137.1, `opt_level=speed` | **331,732** | 317,111–335,667 | 9,516 | **75.0%** |

An independent same-day **Wasmtime 50.0.1** CPU0 control using the same 30k/7×300k dimensions yielded **367,426 H/s** median versus **331,732 H/s** for optimized direct IR. A raw `JITBuilder::new()` policy diagnostic, subsequently found to default to `opt_level=none`, yielded **335,470 H/s**. Both controls are retained in `results/diagnostics/`, not promoted as duplicate canonical leaderboard rows. The unoptimized-to-optimized difference is small and should not be overinterpreted; the more important finding is that **native Cranelift IR is not faster than its Wasmtime/Wasm path here**. The native adapter's measured loop is in Rust calling the JIT entry point per hash; Wasmtime's batch loop is inside the Wasm module, so the roughly 9.7% direct-vs-Wasmtime same-day difference cannot be assigned exclusively to compiler backend code quality. Native `BUILD_US` includes JSON parsing, IR emission and compilation; Wasmtime measures module construction and instantiation. Raw diagnostic data, exact methods and hashes are below.

## EPYC 9655 / Zen 5 — native Cranelift in-JIT batch loop (2026-11-04)

`epyc-9655-zen5-cranelift-batch-2026-11-04.json` is the first accepted CPU0 measurement of `cranelift-batch`: a **separate adapter** that emits a native `bench(start, count) -> sink` Cranelift function whose 64-bit counted loop and XOR accumulator execute *inside the JIT*, calling the **same native HashWX `hash` function** as the already curated `cranelift-direct` adapter. Unlike the per-hash Rust host loop, the generated `bench` function crosses the host/JIT boundary only once per sample. It is deliberately distinct from rewriting or inlining HashWX into the loop, which might change machine-code costs.

The new batch function matched an independent native-host fold **and** the independently emitted Wasm `bench` export for **627 differential cases** (including empty batches, wraparound starts, random ranges, and 300,000 hashes), all outputs byte-for-byte (SHA-256 `90d845d5d8fb5720a0c5e35e0f6194ac555fdddd5c01aa8260c1c9c35a87cb01`). The exact same release executable also passed the existing **100,000-salt single-hash oracle** (SHA-256 `f9a9a50baa9201bd847fc39e8f2b88036883de0a298b322dd46197029a987cc3`). Batch-mode attestation `MODE jit-batch` is required by the harness; the Cranelift version is 0.137.1 with explicit `opt_level=speed`.

The accepted EPYC 9655 CPU0 run uses **30,000 warmup hashes and seven samples of 300,000 hashes**, each with the expected `1842ddbd4c0fefed` sink. This accepted first run has median **315,342.408 hashes/s** (range **294,252.779–320,323.507**), and `BUILD_US=9753.065` (both JIT functions generated and finalized). The accepted source revision is `1a9b92c`; the checksummed native executable was transferred directly from Acer to cold-storage (SHA-256 `252454017ac9a446797a5e71e4dc45b1ff5fa40ab89e5c5d9f2ca163ddd7acc7`).

Two more batch runs and matched fresh controls on the same CPU0, all 30k/7×300k with correct sinks, are in the single `results/diagnostics/cranelift-batch-controls-cpu0-2026-11-04.json` file:

| implementation | run medians, hashes/s | median of run medians, hashes/s |
|---|---|---:|
| Native JIT-internal batch | **315,342**, 323,380, 318,382 | **318,382** |
| Native Rust per-hash host loop (same executable) | 303,469, 321,397 | **312,433** |
| Wasmtime 50.0.1 in-Wasm batch loop | 353,026, 357,376 | **355,201** |

The modest **~1.9%** difference between the median-of-medians for native batch and native direct is *smaller than observed run-to-run variation* and did not hold across every comparison. Therefore, **moving the loop inside native Cranelift does not establish a robust speed improvement**. Wasmtime remains faster in both matched runs; the typical ~12% gap cannot be attributed to Rust/JIT call boundaries alone. Wasmtime may optimize or inline the `hash` call and lower its memory/stack or control flow differently; those mechanisms were not established by this experiment. The accepted result is the first chronological batch run, not a cherry-picked repeat. The six follow-up control records remain diagnostic rather than additional canonical leaderboard rows.

## EPYC 9655 / Zen 5 — native Cranelift branch lowering (2026-11-04)

**Current optimized native Cranelift result:** `epyc-9655-zen5-cranelift-branch-2026-11-04.json` contains *two* accepted CPU0 runs, one `cranelift-batch` and one `cranelift-direct`, compiled with Cranelift **0.137.1**, `opt_level=speed`, source revision **`280af67`**. Both meet **30,000 warmup / seven × 300,000-hash samples**, with all sinks `1842ddbd4c0fefed`. The native binary SHA256 on Acer and cold-storage is `5bf75f0c86a2bee00285a59c6930aba2f7fce6f21b036da487718a3fa32724a9`. The compiled hash passed 100,000 per-input checks against independently emitted Wasm; the JIT-local batch passed 627 independent three-way batch cases, including a full 300k range and wrapping starts.

| Native Cranelift implementation (first accepted run) | Median H/s | First-run range H/s | Difference from the earlier native exhibit |
|---|---:|---:|---|
| `cranelift-batch` after sequential branch exits | **352,504** | 348,924–364,025 | Earlier batch: 315,342 H/s |
| `cranelift-direct` after sequential branch exits | **359,416** | 352,579–368,022 | Earlier direct: 331,732 H/s |

The key change is a **general semantic-preserving improvement in generated control flow**, not a special-cased HashWX result. Previously the native emitter combined two branch conditions into a Boolean-AND before `brif`; Cranelift emitted **124 `setcc` instructions** in the hash function. Emitting sequential early-exit branches instead (as the validated Wasm emitter does) removes every `setcc`; native compiled-hash code falls from **20,488 bytes / 4,213 decoded instructions** to **19,944 bytes / 4,083 instructions**. Both native and Wasmtime use the same 262 BMI2 `rorx` instructions. The actual emitted machine code was extracted from native `JITModule` finalized executable memory using an explicit `--dump-code` flag and from Wasmtime 50.0.1's serialized native ELF (not the Wasm bytecode). The Wasmtime batch function contains a real native call to the hash function, so **inlining was not the explanation** for Wasmtime's initial lead.

For a closer controlled comparison than the two accepted first runs, the **10 other complete CPU0 experiments** (each 30k/7×300k, all checks/sinks correct) are preserved in `results/diagnostics/cranelift-branch-controls-cpu0-2026-11-04.json`. Combining *all* 12 runs' per-run medians, including the two accepted first runs, gives:

| Control group | Independent run medians, H/s | Aggregate of run medians, H/s |
|---|---|---:|
| Original native batch | 329,227; 330,529 | **329,878** (mean of two) |
| Branch-optimized native batch | 352,504; 366,347; 363,055; 364,914 | **363,984** (median of four) |
| Original native direct | 326,160; 300,809 | **313,484** (mean of two) |
| Branch-optimized native direct | 359,416; 348,498 | **353,957** (mean of two) |
| Wasmtime 50.0.1 in-Wasm batch | 359,221; 368,339 | **363,780** (mean of two) |

Branch-optimized batch is **10.3% faster** than its interleaved baseline aggregate; branch-optimized direct is **12.9% faster** than its matched baseline aggregate. The optimized native batch and Wasmtime now differ by **0.06%** in these run-median aggregates—much smaller than observed run variation. **The supported result is throughput parity, not a reliable native win.** The old historical exhibit datasets remain intact, with their original source revisions and provenance. See `results/diagnostics/README.md` for full code-size counts, disassembly and reproduction instructions.
