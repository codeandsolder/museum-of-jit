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

### 2026-11-04 — Generated-code analysis identifies native branch-lowering inefficiency

**Accepted branch-optimized observations:** [`../epyc-9655-zen5-cranelift-branch-2026-11-04.json`](../epyc-9655-zen5-cranelift-branch-2026-11-04.json), SHA256 `876a33a85982bcee91020103d5394e7609d7f6aa7e39ec1e8edc693cdbb0137c`, holds the **first** canonical CPU0 runs of the new native `cranelift-batch` (median 352,504.291 H/s) and `cranelift-direct` (median 359,415.705 H/s), not best-of-N cherry picks. Another **10 complete interleaved CPU0 runs** with every sample, run ID, timestamp, exact source commit and runtime metadata are in [`cranelift-branch-controls-cpu0-2026-11-04.json`](cranelift-branch-controls-cpu0-2026-11-04.json), SHA256 `443c7ed5eb00992d64065afcd5c8fe4c0df4210f8b8ae1eb63bcfdcb0bef39d0`. Baseline source commit `2adcb47`, optimized source commit `280af67`. Original binary SHA256 `252454017ac9a446797a5e71e4dc45b1ff5fa40ab89e5c5d9f2ca163ddd7acc7`; optimized binary SHA256 `5bf75f0c86a2bee00285a59c6930aba2f7fce6f21b036da487718a3fa32724a9`. All runs **30k warmup, seven × 300k, affinity [0], exact fixture `fd5d450b767035c5` and every sample sink `1842ddbd4c0fefed`**. Raw original JSON files are separately retained under `/srv/scratch/museum-jit-runtimes/cranelift-codegen-analysis-20261104/` on cold-storage and transferred to Acer's `/home/jan/build/museum-of-jit/codegen-analysis-20261104/bench/`, with byte-level SHA256 validation before curation.

**Real compiled native code (EPYC 9655; independently checked on Acer):** Wasmtime Python **50.0.1** `Module.serialize()` returned a valid compiled ELF, SHA256 `75040d6bcfd31612aa2a5c27e10b806a11275fe8a19c009a1d8740dcf8c5be40`; Wasm input SHA256 `8e70d5fcf7c85bb3ff64cc55a1811506bf7b067f507b7bd426bd2df0943d3690` (14,166 bytes). Its ELF symbol table reports `wasm[0]::function[0]` (hash) **16,704 bytes**, `wasm[0]::function[1]` (batch) **144 bytes**; `llvm-objdump` shows the batch's explicit `callq` to `wasm[0]::function[0]` at offset **0x418e**. **Wasmtime did not inline hash into batch**, contrary to the original hypothesis. Our native `--dump-code OUTPUT_DIR` now writes the exact finalized relocated bytes of `hash` and `batch`, retrieving byte lengths from Cranelift `Context.compiled_code()` before context clear. These bytes are diagnostic output only, never incorporated into the benchmark code path. The compiled function data are stored outside Git under `/srv/scratch/museum-jit-runtimes/cranelift-codegen-analysis-20261104/` and `/home/jan/build/museum-of-jit/codegen-analysis-20261104/`.

| Actual EPYC 9655 compiled hash | Wasmtime 50.0.1 | Native Cranelift before | Native Cranelift after |
|---|---:|---:|---:|
| Compiled bytes | 16,704 | 20,488 | 19,944 |
| Decoded x86 instructions | 3,888 | 4,213 | 4,083 |
| BMI2 `rorx` instructions | 262 | 262 | 262 |
| `setcc` instructions | 0 | 124 | **0** |
| MOV-class instructions | 1,263 | 1,521 | 1,467 |
| Stack-relative operands | 428 | 667 | 589 |
| Native batch bytes | 144 | 108 | 108 |

Native Cranelift originally emitted `has_budget && branch_bit_clear` as materialized predicates, giving two `setcc` for each of 62 conditional program checks. The equivalent sequential control-flow exits remove the 124 `setcc` instructions and produce a more Wasm-like branch structure. This is supported directly by exact disassembly **and** a robust interleaved native CPU0 A/B throughput improvement (~10.3% batch, ~12.9% direct using aggregates of run medians). The different Wasmtime/native stack counts are not directly comparable: native scratch is an explicit ~16 KiB stack slot (frame `sub $0x4070,%rsp`), while Wasmtime has a small frame (roughly 0x60) and uses its linear-memory allocation for scratch; Wasmtime's linear-memory accesses are not counted as stack-relative. No specific advantage from this memory difference was proven.

**Reproduction:** On the target Zen5 machine, generate the canonical module from `workloads/hashwx-fixed-v1/implementations/wasm-fixed/emit_wasm.py` and use `tools/analysis/export-wasmtime-elf.py --wasm MODULE.wasm --output COMPILED.cwasm` with pinned Wasmtime Python 50.0.1 and `parallel_compilation=False` (the canonical host's setting). Inspect with `llvm-nm -S --size-sort COMPILED.cwasm` and `llvm-objdump --no-show-raw-insn -d COMPILED.cwasm`. For native Rust1.99/Cranelift 0.137.1, run the release binary explicitly with `MUSEUM_PROGRAM_JSON=... binary --dump-code OUTPUT_DIR`, then `objdump -D -b binary -m i386:x86-64 --no-show-raw-insn OUTPUT_DIR/hash.bin`. The native dump contains the exact code bytes after relocation/finalization and a manifest recording real byte lengths. Compile only with the existing managed sccache on Acer; run the native compiled binary on target CPU0 to capture the target machine code, although the exact native output was independently found byte-identical on Acer Ryzen 4700U and cold-storage EPYC 9655 for this version.

**Interpretation:** Two matched fresh Wasmtime control run medians were 359,221 and 368,339 H/s; optimized native batch run medians were 352,504, 366,347, 363,055 and 364,914 H/s. The median of four native-batch run medians (~363,984) and mean of two Wasmtime run medians (~363,780) differ only 0.06%, well within run-to-run noise. **Native Cranelift and Wasmtime are effectively tied on this workload after branch lowering.** Historical unoptimized/native-before-branch datasets remain valid for their exact revisions and are not overwritten. Further per-instruction profiling would require perf authorization (`perf_event_paranoid=3`, `perf` absent on cold-storage), but is not needed to establish this targeted causality.

### 2026-11-04 — LLVM ORC 21.1.8 cross-glibc linking and CPU0 performance controls

- **Accepted LLVM ORC first run:** [`../epyc-9655-zen5-llvm-orc-2026-11-04.json`](../epyc-9655-zen5-llvm-orc-2026-11-04.json), SHA256 `e61456e96b2fe0bfa621208e327af6264a8b62e14d32dd98e9a37645419a4d32`. First run median **291,462.315 H/s**; seven samples **295476.650, 294290.064, 291462.315, 288920.730, 284569.288, 291617.462, 290796.471** H/s. `BUILD_US=398945.170`, `FIRST_US=4.200`, all valid `sink=1842ddbd4c0fefed`, source revision **`f3b4775b68db4d0d4f4c69acf8996aebf4aa6931`**, CPU `[0]`, exact 30k/7×300k. `ENGINE llvm-orc`, `MODE jit-batch`, LLVM **21.1.8**, O2 IR and Aggressive codegen policy explicitly attested, `code_bytes=null` (not measured).
- **Five full follow-up and matched control records** (complete all samples, run IDs, timestamps, runtime metadata, CPU affinity, source revisions) are grouped under [`llvm-orc-controls-cpu0-2026-11-04.json`](llvm-orc-controls-cpu0-2026-11-04.json), SHA256 **`24ccbea5209914e7b92522faafbf70a37f433bf538b3d0df654c1230fe29d879`**. The two LLVM runs used branch commit `f3b4775`, the native Cranelift batch/Wasmtime controls used `main=a555e4d`. Every run passed the exact canonical check and all 42 measured sample sinks were correct. Chronological run-median sequence LLVM ORC **291462.315**, Cranelift batch **349548.035**, Wasmtime **362804.262**, LLVM ORC repeat **295304.274**, Cranelift batch repeat **360699.271**, Wasmtime repeat **360135.156** H/s. Mean of per-run medians: LLVM **293383.2945**, Cranelift batch **355123.6530**, Wasmtime **361469.7090** H/s.
- **Exact original raw JSONs** are retained outside source under `/srv/scratch/museum-jit-runtimes/llvm-orc-21.1.8/`, copied SHA-identically to `/home/jan/build/museum-of-jit/llvm-orc-20261104/bench/`: `llvm-orc-1` SHA `e61456e96b2fe0bfa621208e327af6264a8b62e14d32dd98e9a37645419a4d32`; `cranelift-batch-1` SHA `62f4a16cb173d5e1c81a6f7c5ef40c5910c81ebb4ccff1dae92f0309e10fa4af`; `wasmtime-1` SHA `2e78c8b47fbdeac5011135ca8f476d31fbd2e8530014dcd53284f85e02341bfd`; `llvm-orc-2` SHA `caeeed1f2e1210c71205b16a4e03e9e2b16c754ccaaebde9c55aa6d9dddd22d0`; `cranelift-batch-2` SHA `b179f12923977e0cc7ab3dc1dc382415b5141ba7f5539d56dd9d3d7f0b623741`; `wasmtime-2` SHA `0a78021ab1d2b4f29bf67edf7510a9ebffe41efee1ff7237e1ae7a36ddff925d`. Two full 100,000-input differential tests and two full 627-batch tests on Acer **and** cold-storage EPYC9655 CPU0 passed with exact reference stream SHA256 `f9a9a50baa9201bd847fc39e8f2b88036883de0a298b322dd46197029a987cc3` (individual) and `90d845d5d8fb5720a0c5e35e0f6194ac555fdddd5c01aa8260c1c9c35a87cb01` (batches).
- **Verified portable runtime**: executable `/srv/scratch/museum-jit-runtimes/llvm-orc-21.1.8/bin/llvm-orc-hashwx`, exact Acer/cold-storage SHA256 **`d23599b026441bc1f27efe2d1cd40765ae4c7411af59d2c71cf29feef6a68608`**. Acer system LLVM23.1.4 prototype compiled and passed oracles, but couldn't run on glibc2.41 Debian13 cold-storage because dynamic LLVM23 needs glibc2.44. Switched to the existing pinned LLVM21.1.8 Clang+LLD **matched static archive** toolchain (the LLVM21 archives contain bitcode incompatible with Acer system LLD23). Matching LLD21 required target Debian13 `libxml2.so.2` **only to run the linker** on Acer; copied into the project-specific Acer build folder with SHA256 `8f2643af68a9eba917929046806dabd1848867068d0eee0f6799288a8e886963`. The link itself initially selected Acer's `sinh@GLIBC_2.44` / `cosh@GLIBC_2.44`; resolved by pointing linker `-L` to the unmodified **target glibc2.41 `libm.so.6`** SHA256 `6d567d53e895273ca14a1f9dc164fc6c8d39aed2f60aa46a733c2784228915f3`. Both math functions then require **GLIBC_2.2.5**, highest direct required GLIBC version is **2.34**. Neither target libm nor a foreign libc is bundled or used at runtime; cold-storage `ldd` shows only its installed libc, libm, libstdc++, libz, libzstd, libgcc_s. Runtime deployed without root or system package changes. The first full link attempted to use target libm but the linker search array was accidentally overwritten by the linker-compat path; caught by checking **exact final symbol versions** before transfer, corrected by appending both `-L` paths, and rebuilt successfully. No wrongly linked executable was run on cold-storage.
- **Build and reproduction**: on Acer with existing managed `/usr/local/libexec/sccache-client`, never restart/bypass, environment `MUSEUM_LLVM_CONFIG=/home/jan/build/museum-jit-cpython-3.15.2-jit-toolchain/LLVM-21.1.8-Linux-X64/bin/llvm-config`, `MUSEUM_LLVM_LINK_LIBDIR=/home/jan/build/museum-of-jit/llvm-orc-20261104/compat-libs`, `MUSEUM_LLVM_TARGET_LIBM_DIR=/home/jan/build/museum-of-jit/llvm-orc-20261104/target-glibc241`, invoke `tools/build-llvm-orc.sh /home/jan/build/museum-of-jit/llvm-orc-20261104/llvm-orc-21-debian13`. Check `objdump -T` for maximum symbol versions and `ldd` on the target. In the CPU0 experiment set `MUSEUM_LLVM_ORC_BIN=/srv/scratch/museum-jit-runtimes/llvm-orc-21.1.8/bin/llvm-orc-hashwx` and run `taskset -c 0 uv run --no-project --python 3.13 python tools/bench.py --impl llvm-orc --output /srv/scratch/museum-jit-runtimes/llvm-orc-21.1.8/NAME-cpu0-2026-11-04.json`, with the standardized `MUSEUM_WARMUP=30000`, `MUSEUM_ITERATIONS=300000`, `MUSEUM_SAMPLES=7`. The two independent validation scripts (`tools/validate-cranelift-direct.py` and `tools/validate-cranelift-batch.py`) are engine-neutral despite their historical filenames: pass `--binary .../llvm-orc-hashwx` and pinned Node runtime, with `--count 100000` / `--random-cases 512` respectively.
- **Interpretation:** under this specific compiler policy (LLVM O2 module pipeline and Aggressive codegen), LLVM ORC's 2-run median aggregate is **82.6% of native Cranelift** and **81.2% of Wasmtime**, with approximately **40×** the cold JIT compilation time. No universal compiler ranking is inferred. No native JIT-code size is reported: a statically linked ~46 MiB host executable does not measure the generated functions. The historical LLVM23 Acer-only diagnostic is not a valid CPU0 Zen5 leaderboard measurement.
