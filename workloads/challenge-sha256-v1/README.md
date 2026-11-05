# challenge-sha256-v1 — portable SIMD / RyuJIT

A computational-core reimplementation of the SHA-256, single-block
decimal-nonce proof-of-work path from [challenge-disasterer](https://github.com/codeandsolder/challenge-disasterer).
This is deliberately *not* a port of the HTTP service, six protocol
adapters, GPU support, HashWX/HashX, BLAKE3, or KDFs.

## Fixture and correctness

- SHA256(prefix || ASCII_DECIMAL(nonce)); prefix is 42 ASCII bytes:
  `challenge-disasterer|sha256|` right-padded with ASCII `x`.
- Nine decimal nonce bytes, two fixed-position lane ID digits 10..25 and
  seven variable digits 0000000..9999999.
- One padded 512-bit block: nonce begins at byte 42; the 0x80 pad is at
  byte 51. First independent SHA-256 oracle is first eight digest bytes
  `b4b920f70c0de8df` for nonce `100000000`.
- Groups of 8 or 16 lanes (native SIMD width), exhausting seven-digit
  inner sequences per group. On a given host, all three .NET backends
  enumerate the same exact candidates.
- RyuJIT scalar and portable SIMD kernels perform SHA-256 compression,
  hotstarting after the first 10 invariant rounds. The SIMD version uses
  `System.Numerics.Vector<uint>` only: no unsafe code, ISA-specific intrinsics,
  native dependencies, or hardcoded architecture dispatch.
- Independent BCL SHA256.HashData verifies all 8 digest words over 16
  lane IDs and multiple decimal-carry boundaries. Partial vector batches
  are independently checked against BCL and scalar modes.

## Modes

| Implementation ID | Description |
| --- | --- |
| `dotnet-challenge-portable` | RyuJIT portable vector SHA-256 core, native SIMD width |
| `dotnet-challenge-scalar` | Handwritten C# scalar SHA-256 with same hotstart |
| `dotnet-challenge-bcl` | .NET cryptography library, SHA256.HashData |
| `rust-challenge-simd` | Pinned real Rust production challenge-solvers SIMD engine |

The BCL library benchmark may use native cryptographic acceleration:
it tests .NET's best available library path, not *only* RyuJIT.
The Rust production solver optimizes the hit-only path and omits
digest feedforward words unnecessary for its comparison. It validates
work accounting, whereas the C# modes compute an XOR digest sink.
These are real differences in instruction mix, so the Rust row is a
production reference rather than identical synthetic work.

## Reproduce

~~~bash
uv run --no-project --python 3 tools/bench.py --workload challenge-sha256-v1 --list
MUSEUM_WARMUP=30000 MUSEUM_ITERATIONS=300000 MUSEUM_SAMPLES=7 \
  uv run --no-project --python 3 tools/bench.py \
  --workload challenge-sha256-v1 \
  --impl dotnet-challenge-portable \
  --impl dotnet-challenge-scalar \
  --impl dotnet-challenge-bcl \
  --impl rust-challenge-simd \
  --output /home/jan/benchmarks/museum-of-jit/challenge-sha256.json
~~~

Pin the benchmark to one logical CPU; compare median hashes/s on
the same CPU. `Vector<uint>.Count` is recorded as backend provenance.
The Rust adapter uses the pinned production git revision and
nightly-2026-11-03 (portable SIMD requires nightly) with canonical
sccache. Runners build in ephemeral directories, then clean up.
Portable C# falls back to software vector operations when hardware
SIMD is not available, preserving correctness.
