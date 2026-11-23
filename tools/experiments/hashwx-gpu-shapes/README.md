# HashWX CUDA shape, cache, and occupancy experiments — RTX 3080

Reproducible fixed-seed, oracle-verified CUDA benchmark inspired by
[RRZE-HPC/gpu-benches](https://github.com/RRZE-HPC/gpu-benches), especially its
GPU latency, strides, cache, L2-cache, and streaming occupancy probes.
This is an isolated experiment, **not a production general-purpose HashWX JIT**.

A full HashWX hash creates a ~16 KiB scratchpad of register snapshots, then
executes 1,792 data-dependent, potentially unaligned 64-bit scratchpad reads.
The experiment changes execution geometry, cache placement and lowering without
changing the hash algorithm.

## Reproduce

On RTX 3080 (SM86), CUDA 13.x:

~~~bash
./repro.sh /path/to/explicit/scoped/build-directory
~~~

The script compiles four variants with nvcc -O3 and runs all four against the
same corpus, checking **every hash output against the reference**. Compiled
binaries live only under the explicitly provided build directory.

The CUDA fixture is a generated, expanded **single HashWX program**. The input
and expected binaries contain 4,096 unique nonce/hash pairs taken from a
CPU-reference fixture. For longer batches, those pairs repeat. Thus larger
batches test GPU saturation, **not independent generated seeds**.

Kernel interface, run from the fixtures directory:

~~~text
/path/to/binary NONCE_COUNT CUDA_THREADS_PER_BLOCK SHARED_SPOILER_BYTES SAMPLES
~~~

CUDA events measure kernel-only time after warmup. The normal reproduction
uses nine samples and records the median. It excludes compilation, kernel
module loading, CPU/GPU transfers and program generation.

Compile-time options:
- USE_SHARED=0: 16,392-byte private scratchpad per lane in global memory
- USE_SHARED=1: private scratchpad per active lane in CUDA shared memory
- GROUP_SIZE: number of lanes with word-interleaved global scratchpads;
  GROUP_SIZE=1 is the original thread-contiguous layout
- PITCH_WORDS: scratchpad stride in 64-bit words; minimum 2049
- HASHWX_BRANCHLESS_LOAD=0: original conditional unaligned 64-bit read
- HASHWX_BRANCHLESS_LOAD=1: always load both adjacent 64-bit words, then
  combine with shifts and a mask so the zero-shift case remains correct

The bench/shape-persistent.cu file also reproduces persistent/grid-stride
shared-memory blocks. The logs directory contains timing and correctness
reports from each occupancy, wave-size, interleaving, cache-policy, shared-memory,
persistent-grid, pitch, and branchless-load experiment.

## Measured RTX 3080 performance

All rates below are **millions of full HashWX hashes per second**.
Device: GA102 68 SMs, CUDA 13.1/Windows 591.86 driver from WSL.

| Variant | 4,096 hashes/launch | 65,536 hashes/launch |
| --- | ---: | ---: |
| Global scratch, conditional read, 32 lanes/CTA | 1.93 | 2.66 |
| Global scratch, branchless read, tuned small CTA | 3.01 | 3.18 |
| Shared scratch, conditional read, best CTA | 3.33 | 3.47 |
| **Shared scratch, branchless read, 2 lanes/CTA** | **4.87** | **5.43** |

Shared scratch with branchless read also achieved 5.23 MH/s at 16,384
hashes/launch. At matched 65,536-hash batch size, the improvement is **2.04x**.
On this RTX 3080 the result approaches tevador's published 5.8 MH/s on an
RTX 5060 Ti; that is a **cross-device**, non-identical benchmark comparison.

### Architecture findings

- **Launch saturation:** 4,096 nonces underutilize the device. The original
  global kernel reaches about 2.75 MH/s at 16,384 nonces.
- **Shared memory:** full 16 KiB/hash on-chip scratchpads substantially reduce
  dependent-load overhead even with just 1–3 live lanes per CUDA block.
- **Branchless loads:** always issuing the second word load, even for aligned
  addresses, enables load overlap and avoids divergent control flow.
- **Small CUDA blocks:** 4–8 active lanes often beat a full 32-lane block
  in the global-scratch kernel. More simultaneously active hashes can make
  cache locality worse.
- **Per-thread contiguous beats word-interleaved layouts:** 2–128-lane
  coalescing groups made random scratch reads much less efficient overall.
- **Artificially restricting CTAs** to one resident warp/SM loses throughput;
  some independent memory operations are needed to hide latency.
- **Hybrid half-shared/half-global scratch**, CUDA local stack-based scratch,
  and persistent-CTA launches did not beat full shared memory.
- **Scratch pitch:** the 2,049-word allocation strongly outperforms even small
  amounts of padding for the branchless shared kernel; see the paired runs.
  Larger padding can cross shared-memory residency thresholds.

The original raw benchmark printed fractional "resident_warps_sm" for subwarp
CTAs, which is inaccurate: each CTA consumes at least one complete warp.
The checked-in benchmark source corrects this to the product of resident CTA
count and allocated warps per CTA; timing measurements are unaffected.

### Limits and next step

This experiment only proves fixed-seed kernel throughput. Before choosing
a production backend, test many *independently generated* HashWX programs,
including startup/compilation, solve latency, and correct difficulty search;
record actual L1/L2 traffic, shared-memory bank conflicts, scoreboard stalls
and clock/power counters. Do not interpret streaming cache bandwidth as
dependent random-load throughput.

### Why branch removal and scratch pitch matter

The NVIDIA SASS comparison in logs/sass-compare-20261123.txt explains two
otherwise surprising observations:

- The original shared-scratch kernel has **295 BRA** instructions and
  **289 BSSY/BSYNC pairs** in its static generated program. The branchless
  form has **71 BRA** and **65 BSSY/BSYNC pairs**: **224 conditional read
  branches and their reconvergence regions disappeared**. This is a
  much larger effect than avoiding one simple branch in a hot scalar loop.
- Increasing scratch pitch from 2049 to 2056 words reduces the reported
  register allocation from 52 to 40, but adds **489 static IMAD**
  instructions (2209 to 2698) while removing 152 STS instructions
  (305 to 153). Both retain **3 resident CTAs/SM** at two threads/CTA.
  Five interleaved 65,536-hash trials put pitch2049 around
  **5.1–5.5 MH/s** and pitch2056 around **3.90–3.91 MH/s**.
  The regression is accompanied by significant changes in compiler
  instruction selection, **not just lower occupancy**. Attribution to
  a specific shared-memory bank conflict would require profiler counters.

An independently recompiled run from the exact checked-in source measured
**5.463 MH/s** on the RTX 3080 for 65,536 hashes, with all 65,536 outputs
oracle-verified (see logs/reproduction-20261123.log).

## Multi-seed source generation and compilation-inclusive measurements (2026-11-23)

A Rust [IR-to-CUDA generator](generator/src/main.rs) emits **all 13**
HashWX instruction types from the architecture-neutral hashwx-rs program
representation, including the exact 32-subprogram branch policies,
seed-derived SipHash key, and both phases. It independently generates
native-CPU-JIT reference hashes for any requested number of distinct nonces.

The standalone [NVRTC runner](bench/multiseed_nvrtc.cu) compiles one complete
seed-specific CUDA kernel to SM86 CUBIN using the CUDA runtime compiler,
loads it via the CUDA driver API, and checks every output with the CPU oracle.
It separates host program generation, CPU JIT construction, NVRTC compiler
time, CUDA module load, and warmed CUDA-event kernel timings.

### Reproduce

On Acer, with the repository's managed Cargo/sccache configuration:

    cargo +nightly-2026-11-22 test --offline --locked \
      --manifest-path generator/Cargo.toml \
      --target-dir /home/jan/build/hashwx-shapes/generator-target
    cargo +nightly-2026-11-22 run --offline --release --locked \
      --manifest-path generator/Cargo.toml \
      --target-dir /home/jan/build/hashwx-shapes/generator-target \
      -- /home/jan/build/hashwx-shapes/seeds 16 65536

Transfer the generated seeds directory to a CUDA machine. Build and run:

    /usr/local/cuda/bin/nvcc -O2 -std=c++17 -arch=sm_86 \
      bench/multiseed_nvrtc.cu -o /path/to/build/multiseed_nvrtc -lnvrtc -lcuda
    /path/to/build/multiseed_nvrtc /path/to/generated/seeds 7

The generator output directory and CUDA binary build directory are explicit,
outside the checkout. No CUDA binary or generated seed corpus is committed.
Raw measurements are stored in logs/.

### Physical RTX 3080 results — 16 independent programs

- **16 separately generated programs, 65,536 distinct nonces each**:
  1,048,576 distinct program/nonce pairs, plus smaller batch repeats, with
  **every GPU output verified** against an independent native CPU-JIT oracle.
- CPU-side source generation on Acer ~0.25–0.5 ms/seed usually; native CPU JIT
  compilation **0.05–0.17 ms/seed**. Oracle hashing duration was affected
  by other work on Acer, so it is not a clean CPU throughput measurement.
- Eight previously compiled seeds: NVRTC compile **34–41 ms**.
  Eight new seed kernels: NVRTC compile **1.65–2.40 seconds**. Reusing an
  identical source benefits from compiler caching; initial application-wide
  driver initialization is a separate variable.
- CUDA module load **0.27–0.60 ms**.
- With a large batch first to warm the GPU, 463-hash kernels execute in
  **0.151–0.171 ms** across 16 seeds; 4,096-hash kernels take
  **0.795–0.896 ms**. Earlier 5x short-batch outliers followed launch
  order/power-state effects, not intrinsic seed complexity.
- At 65,536 distinct nonces, **15 of 16** seeds delivered roughly
  **4.76–5.47 MH/s**; seed #13 unexpectedly reached **2.23 MH/s**
  median while its smaller batches were normal. This needs an isolated
  retest before being treated as an architectural outlier.
- Cold NVRTC compilation is vastly slower than native CPU JIT construction
  for a new proof with approximately 463 candidate hashes. The optimized GPU
  kernel is mainly useful if compilation is amortized or compilation is made
  dramatically cheaper.

The new corpus uses distinct nonces, not repeated 4,096-case fixtures.
See logs/multiseed-16-unique-nvrtc.log and earlier order/cache controls.
A static GPU interpreter or a direct machine-code emitter could eliminate
heavy NVRTC compilation; this benchmark is only for specialized CUDA kernels.
