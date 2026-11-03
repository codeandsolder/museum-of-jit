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
