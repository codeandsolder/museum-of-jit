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

## CPU-pinned performance validation

Builds and benchmarks should not be conflated. Pin the main runner to
one logical CPU, but allow toolchain builds to use an explicit, wider
CPU set using `MUSEUM_BUILD_CPUSET`. The actual managed and native
benchmark processes inherit the single-core affinity:

~~~bash
MUSEUM_BUILD_CPUSET=0-7 MUSEUM_WARMUP=1000000 \
MUSEUM_ITERATIONS=10000000 MUSEUM_SAMPLES=7 \
taskset -c 0 uv run --no-project --python 3 tools/bench.py \
  --workload challenge-sha256-v1 \
  --impl dotnet-challenge-portable \
  --impl dotnet-challenge-scalar \
  --impl dotnet-challenge-bcl \
  --impl rust-challenge-simd \
  --output /home/jan/benchmarks/museum-of-jit/challenge-sha256-acer-cpu0.json
~~~

On AVX-512 systems, execute a second, separately recorded run with
`DOTNET_MaxVectorTBitWidth=512`. The JIT's actual lane count and this
override are both stored under the result's runtime metadata. Without
that override the same .NET 10 portable source can select eight rather
than sixteen u32 lanes despite AVX-512 support.

Where Rust is unavailable on the benchmark host, compile once on the
development machine using canonical sccache and a compatible target,
then set `MUSEUM_CHALLENGE_RUST_BIN` to the executable and
`MUSEUM_CHALLENGE_RUSTC_VERSION` to the exact rustc version. The
runner records the executable's SHA-256. The provided executable
must implement the pinned source and pass the fixed SHA-256 oracle;
a successful checksum is necessary but not sufficient to establish
source provenance. No compilation is performed when supplied with a
prebuilt binary.


## Accepted Zen 5 results

See [curated result notes](../../results/README.md) and the tracked
`epyc-9655-zen5-challenge-sha256-{256,512}-2026-11-05.json` records.

On EPYC 9655 CPU0 with .NET 10.0.403, 1M warmup and seven × 10M
candidates: portable C# SIMD reaches 29.81 MH/s with eight lanes,
58.44 MH/s with sixteen lanes (`DOTNET_MaxVectorTBitWidth=512`);
the pinned Rust AVX-512 production solver reaches 118.17 MH/s.
Scalar C# is 6.15 MH/s with the same 512-bit JIT policy.
The C# modes' per-sample XOR digests match exactly within each
run. Changing lane width changes the candidate enumeration order
and hence the sink, not the hash work per candidate.

A fully unrolled C# SHA-256 compression experiment compiled correctly
and passed the oracle but was 3.1× slower than the ordinary loop on
Acer CPU4 (6.6 vs 20.5 MH/s). It was rejected rather than adding
a slower and much larger active implementation.

## Equal-target match-control benchmark

The `dotnet-challenge-match` backend uses the existing portable SIMD
compression kernel but checks the same 64-bit exact-match target as
`rust-challenge-simd`: `0xbaddcafe12345678` (high/low digest words).
Neither emits a per-candidate digest XOR. Both report the exact number
of attempted candidates as the sink; the harness rejects wrong counts.
The full-digest SHA-256 BCL oracle, decimal-carry checks, and partial-lane
tests run before either .NET benchmark mode.

This isolates the effect of replacing digest-XOR reduction with
match-only control flow, but **does not make their SHA implementations
instruction-identical**: the C# kernel still calculates all eight
feedforward digest words while production Rust only needs two for
64-bit target matching. The original `dotnet-challenge-portable` and
all curated historical results remain unchanged and directly comparable
by source revision and fixture hash.

The production solver enumerates a different prefix range after
exhausting each full inner sequence. Benchmarks up to 10 million
candidate hashes run only the initial prefix group, so all match
the same lane IDs and decimal inner indices. The C# match mode
also correctly handles partial vector batches; any unexpected hit
inside the valid lane range aborts rather than silently counting it.


## AVX-512 RyuJIT codegen and control findings (2026-11-05)

The measured **match-control** compares the same fixed 64-bit SHA-256
equality target as the production Rust solver and checks the exact
attempted candidate count. On CPU0 of an EPYC 9655, with 16 lanes,
1M warmup + 7×10M, production Rust SIMD achieved 121.86 MH/s,
portable C# target-match 59.83 MH/s, and existing C# digest-XOR
61.57 MH/s. The paired comparison confirms the gap is **not**
caused by XOR sink reduction. Original backend/results remain intact.

In the genuine RyuJIT `CompressVector` disassembly, portable
`Vector<uint>` compiles to AVX-512 `zmm` operations, but vector rotates
lower to multiple shifts/ORs, unlike LLVM's unrolled production Rust
kernel using `vprold`. RyuJIT's 1,149-byte compression also zeros
a 1,024-byte stack message ring and calls a memmove before rounds;
both are absent in the same form from Rust's 15.3 KiB specialized
unrolled function. Separate scratch-buffer, no-init (requires /unsafe),
and fully-unrolled C# attempts regressed on Acer. Moving the first six
rounds into a separate loop matched baseline on EPYC in controlled
A/B checks (no measurable net gain); keep the original simpler loop.
Acer CPU4 comparison is diagnostic only due to pronounced frequency
or concurrent load variability.

Collected disassembly/AB diagnostics live outside the repository under
`/srv/scratch/benchmarks/museum-of-jit/diagnostics/codegen-20261105/`.
Curated EPYC raw samples are in
`results/epyc-9655-zen5-challenge-sha256-match-2026-11-05.json`;
Acer's raw diagnostic is under `results/diagnostics/`.


## Portable unrolled C# experiment: stable AVX-512 performance (2026-11-05)

The new `dotnet-challenge-optimized` (digest-XOR) and
`dotnet-challenge-match-optimized` (64-bit hit-only) backends use a
**source-generated**, 16-word, fully unrolled SHA-256 compression function
expressed entirely in portable `System.Numerics.Vector<uint>` operations.
No architecture-specific intrinsics, unsafe code, or dynamic code generation
are required. The canonical compact loop and old backends are unchanged.
The optimization is selected **only when `Vector<uint>.Count == 16`**:
on AVX2/other narrower targets the new modes deliberately use the old
portable implementation. On AVX-512-capable .NET 10, enable
`DOTNET_MaxVectorTBitWidth=512` to select the sixteen-lane path.

The generated file `Sha256Kernel.Unrolled.g.cs` is **mechanical output**
from `tools/generate_challenge_sha256_unrolled.py`, a small development/CI
source generator deriving round logic and both drivers from the canonical
source. Run `uv run --no-project --python 3
tools/generate_challenge_sha256_unrolled.py --check` to detect drift.
This isolates the lengthy expansion without maintaining duplicate business
logic. The full-digest RustCrypto-independent BCL self-test checks all
eight words of **both** compression implementations for every lane and
decimal-carry case, plus partial candidate batches in both new modes.

**Important JIT finding:** a fully unrolled C# method suffers pathological
Tier-0 performance when .NET 10's default tiered compilation is enabled:
some samples fell to ~10 MH/s. The narrow
`[MethodImpl(MethodImplOptions.AggressiveOptimization)]` hint on the
generated compression function bypasses Tier 0 *for that method only*,
without disabling tiering or PGO process-wide. It also bypasses
dynamic-PGO benefits for this one function; do not apply it indiscriminately.
Full unroll on 8-lane Zen 2 was previously slower due to register pressure,
hence the explicit width-16 dispatch above.

Final paired Zen 5 CPU0 A/B validation, **default runtime tiering,
512-bit portable vectors, 1M warmup, 2 × 7 × 10M** independent
candidates per mode, source built on Acer with .NET 10, executed
on cold-storage with .NET 10.0.403:

| C# mode | Existing compact loop | Optimized full unroll | Gain |
| --- | ---: | ---: | ---: |
| Exact 64-bit target-match | 57.10 MH/s | **62.70 MH/s** | **+9.8%** |
| Digest-XOR reduction | 60.09 MH/s | **65.75 MH/s** | **+9.4%** |

Every run passed the known SHA-256 fixture; both target-match modes
produced exactly `0000000000989680` attempted-count sinks, with
matching digest-XOR sinks between old/new modes. Additional 7×10M
and 3×7×10M experiments confirmed the unrolled version stabilizes
around 64–67 MH/s with tiering disabled or a method-level
optimization hint. The 2026-11-05 negative experiments include:
fixed `Vector512<uint>` (no meaningful advantage over 512-bit
`Vector<uint>`), alternative Ch/Maj expressions (~1–2% noise),
16 explicit message-schedule copies, and 4-/8-round partial
unrolls (no reproducible improvement). Disabling ReadyToRun
actually decreased throughput by about 20%.

The complete measured per-sample diagnostics, including rejected
experiments and JIT policy controls, live in
`results/diagnostics/csharp-sha256-ryujit-20261105/`. The production Rust
AVX-512 SHA-256 matcher was previously ~121.86 MH/s on this host:
the optimized C# matcher now reaches **~51.5%** of Rust, versus
~46.9% with the compact C# loop in the paired final run.
These remain *different full-digest calculation mixes*, as documented
in the original workload comparison. The gain here is an internal
C#-versus-C# comparison on the same work and runtime.
