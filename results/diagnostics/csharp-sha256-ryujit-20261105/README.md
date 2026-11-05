# .NET 10 portable SIMD SHA-256 optimization trials — 2026-11-05

This folder retains *raw per-sample* measurements from CPU0 of
cold-storage EPYC 9655, .NET 10.0.403, with
`DOTNET_MaxVectorTBitWidth=512`, 1M warmup and 7×10M candidates
per process. All variants passed independent BCL full SHA-256 digests
and exact target-matching nonce counts. Alternating forward/reverse
execution order was used to reduce ambient host load bias.

- `first.json`: original `Vector<uint>`, simplified Ch/Maj,
  fixed-width cross-platform `Vector512<uint>`, and combination.
- `second.json`: explicit 16-word schedule copy, 4-/8-round unrolling,
  and their combination.
- `boolean.json`: Ch/Maj each isolated and alternate OR Ch formulation.
- `unroll.json`: full in-register, compile-time round expansion **without**
  JIT optimization hint; default tiering has severe performance cliffs.
- `runtime.json`: default compiler/JIT policy versus tiered compilation off,
  dynamic PGO off, quick-loop tiering off and ReadyToRun off.
- `unroll-tiering.json`: full unroll and compact loop with default
  tiering versus `DOTNET_TieredCompilation=0` (stable full unroll).
- `aggressive.json`: method-only `AggressiveOptimization` on
  compact, 8-round unrolled, and fully unrolled compressors.
- `final.json`: accepted paired A/B of *final generated production
  experiment* versus original portable C#: **+9.8% target-match
  (57.10→62.70 MH/s) and +9.4% digest-XOR
  (60.09→65.75 MH/s)**, default .NET tiering.

The only implemented optimization is a generated, fully unrolled
portable vector compressor with a method-specific JIT optimization hint
and sixteen-lane-only runtime selection. Existing workloads and mode
results remain unchanged. Avoid promoting sub-2% differences from the
negative trials; some host runs were noisy. The mechanical generated
C# is maintained by `tools/generate_challenge_sha256_unrolled.py`;
it does not introduce a runtime or production Python dependency.

See [workload documentation](../../../workloads/challenge-sha256-v1/README.md).
