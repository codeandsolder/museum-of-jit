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
