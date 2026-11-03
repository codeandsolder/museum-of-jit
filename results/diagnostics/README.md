# Diagnostic runs (not the canonical leaderboard)

These are valid harness runs used to investigate benchmark behavior; they do **not** replace the uniform 30,000-warmup canonical leaderboard.

## RPython/PyPy JitDriver warmup sweep — 2026-11-03

Host: cold-storage, AMD EPYC 9655 (Zen 5), Debian 13, logical CPU 0. Binary: RPython 8.0.1 translated `hashwx-rpython-jit-4666c1d`, repository revision `60f1c0d`. All three runs passed correctness (`fd5d450b767035c5`) and retained the canonical sink (`1842ddbd4c0fefed`) for every one of their seven 300,000-hash samples.

| Raw data | Warmup hashes | Median H/s | First sample H/s | Full sample series H/s |
| --- | ---: | ---: | ---: | --- |
| `rpython-warmup-300k-2026-11-03.json` | 300,000 | 74,899 | 71,271 | 71,271, 73,590, 74,899, 74,248, 74,989, 77,034, 77,121 |
| `rpython-warmup-1m-2026-11-03.json` | 1,000,000 | 76,833 | 76,609 | 76,609, 77,180, 76,833, 78,686, 78,348, 71,573, 73,844 |
| `rpython-warmup-control-30k-2026-11-03.json` | 30,000 | 76,990 | 65,264 | 65,264, 71,331, 75,376, 77,007, 76,990, 77,884, 77,837 |

Original accepted 30k-warmup canonical row: 75,356 H/s median in `../epyc-9655-zen5-2026-11-03.json`.

**Conclusion:** a longer warmup markedly reduces the *first-sample cold-JIT ramp* but does not improve the steady median (~75–77 kH/s). The 1M warmup run still had late dips. This is compatible with the previously measured 8,969 tracing bridges, but this sweep alone does not apportion the time to bridge compilation versus execution. Keep canonical comparisons at uniform warmup until a cross-runtime measurement policy is established.

To reproduce, run on cold-storage's CPU 0 using the same binary and checkout, varying only the warmup and output name:

```sh
MUSEUM_RPYTHON_HASHWX=/srv/scratch/museum-jit-runtimes/rpython-8.0.1/hashwx-rpython-jit-4666c1d \
MUSEUM_WARMUP=300000 MUSEUM_ITERATIONS=300000 MUSEUM_SAMPLES=7 \
taskset -c 0 uv run --no-project --python 3 tools/bench.py \
  --impl rpython-jitdriver --output results/diagnostics/rpython-warmup-300k-2026-11-03.json
```

Raw JSON preserves individual samples, revision, affinity, sink, elapsed time and runtime provenance.

## CPython 3.15.2 Clang21 JIT pathology — 2026-11-03

Host: cold-storage AMD EPYC 9655 / Debian 13, CPU affinity exactly `[0]`. Acer-built runtime copied with matching executable SHA256 into `/srv/scratch/museum-jit-runtimes/cpython-3.15.2-clang21-pgo-r2`; compiler Clang 21.1.8, LLVM native InstrProf PGO, full LTO, audited LLD 23.1.4. The r2 build completed 43 Python PGO training files (9,709 tests run, 462 skipped), confirmed actual tier-2 executor execution, relocation, and maximum referenced GLIBC 2.38.

All runs used the identical generated Python HashWX implementation. Correctness check `fd5d450b767035c5` passed; both long-running experiments generated the same sink `2041e16568b783d8` in all seven samples.

| Diagnostic file | JIT | Warmup | Samples | Median H/s | Full sample series H/s |
| --- | --- | ---: | --- | ---: | --- |
| `cpython-clang21-pilot-2026-11-03.json` | On, 66 installed executors | 300 | 1 × 3,000 | 975.832 | 975.832 |
| `cpython-clang21-warm30k-7x10k-2026-11-03.json` | On, 204 installed executors at end | 30,000 | 7 × 10,000 | 458.485 | 572.634, 537.340, 509.105, 458.485, 434.908, 417.647, 415.612 |
| `cpython-clang21-jit-disabled-warm30k-7x10k-2026-11-03.txt` | Off (same binary) | 30,000 | 7 × 10,000 | 1,185.772 | 1,157.389, 1,169.645, 1,189.112, 1,191.196, 1,185.772, 1,189.272, 1,172.450 |

At the same 30k/7×10k configuration, **turning the JIT off was 2.59× faster by median**. JIT throughput declined 27.4% from its first to last measured sample, while the non-JIT throughput remained comparatively stable. The number of installed executors suggests aggressive specialization, but the result does **not** by itself establish whether the decline comes from code generation, side exits, tracing, cache behavior, or work done by executors. Investigate before proposing a JIT policy change.

The `3k`/`10k` sample configurations are diagnostics, *not* the canonical uniform 30k warmup / 7×300k benchmark. Extrapolating the latter from the declining rates would be misleading; no CPython leaderboard row has been accepted. The baseline control is raw stdout rather than a harness JSON because the ordinary orchestrator intentionally rejects CPython JIT-disabled runs as invalid for its `cpython-jit` implementation.

Reproduce the JIT-enabled run on cold-storage:

```sh
MUSEUM_CPYTHON=/srv/scratch/museum-jit-runtimes/cpython-3.15.2-clang21-pgo-r2/bin/python3.15 \
PYTHON_JIT=1 MUSEUM_WARMUP=30000 MUSEUM_ITERATIONS=10000 MUSEUM_SAMPLES=7 \
MUSEUM_RUN_TIMEOUT=440 taskset -c 0 uv run --no-project --python 3.13 \
  python tools/bench.py --impl cpython-jit
```

Reproduce the same-binary JIT-disabled control:

```sh
PYTHON_JIT=0 MUSEUM_WARMUP=30000 MUSEUM_ITERATIONS=10000 MUSEUM_SAMPLES=7 \
taskset -c 0 /srv/scratch/museum-jit-runtimes/cpython-3.15.2-clang21-pgo-r2/bin/python3.15 \
  workloads/hashwx-fixed-v1/implementations/python-generated/bench.py --engine cpython
```

### Installed CPython executor growth (diagnostic PR #12)

`cpython-clang21-executor-trace-warm30k-7x5k-2026-11-03.txt` repeats the same CPU0 correctness-checked generated source with 30k warmup, 7 × 5k measured hashes, and **opt-in** `--trace-executors`. The additional `get_executor` enumerations occur **after** warmup and **after** each timed sample, never inside the timed intervals. The program source and core benchmark execution remain unchanged. This run used the exact tracer source revision `44bb521`; the temporary cold-storage fixture was removed afterward.

| Stage | Installed executors in `hash_one` | H/s during preceding sample |
| --- | ---: | ---: |
| after warmup | 81 | — |
| after sample 1 | 86 | 553.443 |
| after sample 2 | 89 | 548.782 |
| after sample 3 | 94 | 523.987 |
| after sample 4 | 103 | 537.517 |
| after sample 5 | 109 | 533.417 |
| after sample 6 | 120 | 492.193 |
| after sample 7 | 126 | 447.567 |

Every sample had identical sink `e2f0befa35d3dd61`; the canonical correctness probe passed. Executor count rose **45 (+55.6%) after warmup** while throughput fell ~19% from first to last sample. This demonstrates continued executor installation into the measured function and coincident throughput degradation, but **does not yet quantify compilation cost, side exits, or the effectiveness of individual executors**. Because samples are only 5k long, this is not directly a throughput match for the 7×10k control above or a canonical leaderboard entry.

Reproduce with the accepted Clang21 runtime on cold-storage's CPU 0 and repository source containing `--trace-executors`:

```sh
PYTHON_JIT=1 MUSEUM_WARMUP=30000 MUSEUM_ITERATIONS=5000 MUSEUM_SAMPLES=7 \
taskset -c 0 /srv/scratch/museum-jit-runtimes/cpython-3.15.2-clang21-pgo-r2/bin/python3.15 \
  workloads/hashwx-fixed-v1/implementations/python-generated/bench.py \
  --engine cpython --trace-executors
```

### Side-exit initial threshold experiment (same 30k/7×5k protocol)

The default CPython 3.15.2 `PYTHON_JIT_SIDE_EXIT_INITIAL_VALUE=4000` was compared with its maximum supported setting, `8191`, on the same cold-storage CPU0 and accepted runtime. The raw tuned run is `cpython-clang21-side-exit-max-warm30k-7x5k-2026-11-03.txt`. All seven tuned samples passed the identical `e2f0befa35d3dd61` timed sink and the canonical correctness check.

| 30k warmup + 7×5k | Default threshold 4000 | Max threshold 8191 |
| --- | ---: | ---: |
| Median hashes/s | 533.417 | **615.840** |
| Installed executors after warmup | 81 | 69 |
| Installed executors after last sample | 126 | 78 |
| New installed executors during samples | 45 | 9 |
| First/last sample hashes/s | 553.443 / 447.567 | 702.217 / 571.212 |

The higher threshold improves the median by **15.45%** and substantially slows executor population growth, consistent with side-exit-driven compilation contributing to overhead. It does **not** establish how much of the remaining throughput penalty comes from compilation vs running JIT executors; even the tuned JIT remains below the same-binary non-JIT control median of 1185.772 H/s (measured at 7×10k, so treat rates as a diagnostic comparison, not a matched 7×5k experiment).

Reproduce with `PYTHON_JIT=1 PYTHON_JIT_SIDE_EXIT_INITIAL_VALUE=8191` added to the 30k/7×5k `--trace-executors` invocation above. Investigation: [issue #13](https://github.com/codeandsolder/museum-of-jit/issues/13).

### 2026-11-03 — Direct Cranelift native IR versus Wasmtime on Zen 5

- Accepted speed-optimized native result: [`../epyc-9655-zen5-cranelift-2026-11-03.json`](../epyc-9655-zen5-cranelift-2026-11-03.json), SHA-256 `5fd50ca2a8a2212652e85bf7d02a920ca5e1d70f380ecf95a7c3fa44c027b6fd`; branch commit `23f8043`, CPU0, 30k warmup/7×300k, median **331,731.579 H/s**; seven valid sinks `1842ddbd4c0fefed`; build **9,515.572 µs**. Optimizer explicitly configured `opt_level=speed`; released executable SHA-256 `139a5efcb2bc4e03dff41263e9ad2da09ca8fa530063a926f9cc9ab93a5afde3` verified both on Acer and after transfer to cold-storage.
- Direct-IR **unoptimized default** diagnostic [`cranelift-native-default-none-cpu0-2026-11-03.json`](cranelift-native-default-none-cpu0-2026-11-03.json), SHA-256 `220d8ab9946b703d3dc2f53463741e45d5fd7d299d937f86ef4e9a02be0a2c87`; branch commit `0f5d9a9`, CPU0, same dimensions, median **335,470.141 H/s**, build **6,588.306 µs**. `cranelift-codegen` 0.137.1's default settings show `opt_level="none"`; this is a policy diagnostic, not the chosen canonical result.
- Independently repeated **Wasmtime 50.0.1** targeted CPU0 control [`cranelift-wasmtime-control-cpu0-2026-11-03.json`](cranelift-wasmtime-control-cpu0-2026-11-03.json), SHA-256 `651d4c5017c758d82821000e807663d1dca2f12ad4bbd73ebddf128d662a5e93`; median **367,426.140 H/s**, with valid canonical sinks. This does not replace the already accepted earlier Wasmtime run.
- Before CPU0 timing, the accepted native JIT's `--hash-salts` interface produced exact per-salt agreement with a separately emitted fixed-program Wasm module on **100,000 edge/sequential/random salts**. Deterministic inputs generated with `uv` Python and `random.Random(0xC3A7E517)`; `tools/validate-cranelift-direct.py` and `tools/validation/cranelift-wasm-oracle.mjs` reproduce the test. Both outputs SHA-256 `f9a9a50baa9201bd847fc39e8f2b88036883de0a298b322dd46197029a987cc3`. All test and build artifacts were generated outside the tracked project directory. The speed-optimized native and unoptimized native binary each passed the same 100k test.
- Caution: the direct native entry point is invoked once per hash by a Rust host loop; Wasmtime exports an internal batch loop in Wasm. Do not attribute observed throughput differences exclusively to frontend or code-generation performance without isolating the call boundary.

### 2026-11-04 — Native Cranelift JIT-internal batch versus host-loop and Wasmtime

- Accepted native internal-batch result: [`../epyc-9655-zen5-cranelift-batch-2026-11-04.json`](../epyc-9655-zen5-cranelift-batch-2026-11-04.json), SHA256 `4cc2a40ffe38e4cb762f60ad3eafafedd0fbf863ff56019692b0de05e6a1ba30`. Adapter `cranelift-batch`, Cranelift **0.137.1**, `opt_level=speed`, source commit **`1a9b92c`**, EPYC 9655 **CPU0**, **30,000 warmup**, **seven × 300,000**. All checks `fd5d450b767035c5` and sinks `1842ddbd4c0fefed`, `runtime.jit_batch_internal=true`. Rates **316,186.538, 315,342.408, 313,825.894, 316,362.889, 320,323.507, 304,986.900, 294,252.779 H/s**, median **315,342.408 H/s**; `BUILD_US=9753.065`, `FIRST_US=6.270`. The transferred executable SHA256 on both Acer/cold-storage was `252454017ac9a446797a5e71e4dc45b1ff5fa40ab89e5c5d9f2ca163ddd7acc7`.
- **Six complete targeted control runs**, each preserving all seven raw sample rates, timestamps, `run_id`, JIT/runtime metadata and CPU affinity, are aggregated in [`cranelift-batch-controls-cpu0-2026-11-04.json`](cranelift-batch-controls-cpu0-2026-11-04.json), SHA256 `c39c27998316c06c64c6609e76ef0db04b8499be7fb0bdeaa792f940bc367776`. All runs use `1a9b92c` with **CPU affinity [0], warmup 30k, seven ×300k, same checksum and every sink correct**. Chronological sequence: batch first (accepted), direct control 303,469.474, Wasmtime 353,025.645, batch repeat 323,379.548, direct repeat 321,396.914, Wasmtime repeat 357,375.667, batch third 318,382.270 H/s. Raw files also remain outside Git on cold-storage at `/srv/scratch/museum-jit-runtimes/cranelift-native-0.137.1-batch-20261104/`, with individual SHA256s and cached executable; aggregation changes neither run's provenance nor individual sample arrays.
- Correctness: `uv run --no-project --python 3.13 python tools/validate-cranelift-batch.py --binary /path/to/museum-hashwx-cranelift-direct --random-cases 512` produces **627** independently checked cases (110 edge start/count combinations +512 random +5 special cases), including empty batches, 64-bit index wraparound, and a 300k batch. Exact SHA256 of the native and Wasm result streams `90d845d5d8fb5720a0c5e35e0f6194ac555fdddd5c01aa8260c1c9c35a87cb01`; within each native batch result, the diagnostic also compares a host fold of the individually JIT-hashed range. Existing `tools/validate-cranelift-direct.py --count 100000` still passes on the same executable, SHA256 `f9a9a50baa9201bd847fc39e8f2b88036883de0a298b322dd46197029a987cc3`.
- Interpretation: the native JIT batch loop contains an explicit **native call** to the Cranelift-generated hash; this matches Wasm's batch function structure at the source level, but not necessarily codegen inlining. The ~2% cross-run-median difference between native host and native batch is smaller than observed CPU0 variation; **no robust host/JIT overhead speedup is established**. Wasmtime remained faster in two matched CPU0 runs (~353–357k H/s). Do not convert these diagnostics into additional curated leaderboard rows or attribute the residual gap to WebAssembly alone without machine-code/profiling evidence.
