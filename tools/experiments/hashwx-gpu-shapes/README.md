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

## Static interpreter versus per-seed NVRTC JIT (2026-11-23)

For new, short PoW puzzles, raw 65k-hash throughput is misleading: a
seed-specialized CUDA compiler needs compilation on every previously unseen
program. We therefore recovered the previously verified generic HashWX CUDA
interpreter from **hashwx-rs PR #17** (archived commit
a3ab135d7385d7d16c7b822ee40860aec2baa8f6), preserving its original
portable-Rust oracle generator. The branch has since been removed, but GitHub
still exposes the immutable original source via refs/pull/17/head.

The generic interpreter stores the 256-instruction packed program in shared
memory and uses an independent 16,392-byte **global** scratchpad per active
CUDA lane. A single compiled GPU kernel handles many distinct HashWX
ProgramSets without re-running NVRTC.

### RTX 3080, multiple different programs

Compiled the original interpreter with nvcc -O3 -arch=sm_86, generated 1,024
different programs, checked the 180-seed x 4-nonce baseline oracle (720/720),
and checked the first output for all active programs at every launch shape.
Each active lane executes **463 sequential nonces for its own program**.
The timed result is an XOR of those 463 hashes; all 463 individual hashes per
lane were **not separately verified** in this multiseed interpreter test.

| CTA count | Active lanes / CTA | Concurrent programs | Throughput MH/s |
| ---: | ---: | ---: | ---: |
| 90 | 2 | 180 | 0.182 baseline, 0.188 branchless |
| 136 | 6 | 816 | 0.373 |
| 204 | 3 | 612 | 0.388 |
| 340 | 2 | 680 | 0.516 |
| 256 | 4 | 1024 | 0.545 |
| 320 | 3 | 960 | 0.573 |
| **340** | **3** | **1020** | **0.615** |

The final 340 x 3 configuration repeated at **0.608, 0.615, 0.614 MH/s**
using separate runs, roughly **3.3x the initial 90 x 2 configuration**.
It has about 16 MiB of simultaneous global scratchpads, well beyond
the RTX 3080's 5 MiB L2. This is evidence that more independent data-dependent
loads/latency hiding outweigh higher cache residency for this interpreter.

Changing the interpreter's unaligned load to the same branchless lowering as
the specialized CUDA kernel improves the initial geometry only ~3%, rather
than doubling throughput. Dynamic opcode and register-file handling account
for a larger portion of its cost. Unlike the specialized kernel, placing full
scratchpads into shared memory also **regresses** due reduced concurrency.

### RTX 3080, a *single* previously unseen program

For an apples-to-apples short-challenge latency test, the archived interpreter
was patched to broadcast one ProgramSet to every active lane, assign a unique
nonce to each lane, and verify the resulting digest for **every** output against
1,024 independent native-Rust reference hashes. This is a distinct workload
from the multi-program XOR benchmark above.

| Shape for a 463-nonce search | GPU kernel time |
| --- | ---: |
| 15 CTAs x 32 active lanes | 3.122 ms |
| 58 CTAs x 8 active lanes | 2.123 ms |
| 116 CTAs x 4 active lanes | 1.597 ms |
| 155 CTAs x 3 active lanes | 1.408 ms |
| **232 CTAs x 2 active lanes** | **1.167 ms** |
| 463 CTAs x 1 active lane | 1.315 ms |
| Shared scratch, best tested shape | ~1.903 ms |

The single-program kernel is already compiled and its reported CUDA time
excludes uploading the new ProgramSet and transferring results. It nonetheless
demonstrates that interpreter execution for a ~463-nonce fresh challenge
is ~7 times slower than the specialized post-compilation CUDA kernel
(~0.15–0.17 ms), before accounting for unavoidable host/device overhead.

### Compilation cost and backend policy

Physical RTX3080 NVRTC for **8 new unrelated seeds**: 1.65–2.40 **seconds**
per complete program; **8 previously compiled seeds**: ~34–41 **milliseconds**
per seed, showing the compiler cache matters. CUDA module load ~0.3–0.6 ms.
The native CPU JIT compiles each program on Acer in roughly
0.05–0.17 ms, excluding CPU workload execution. A fresh tiny challenge
therefore should **not** select seed-specialized CUDA/NVRTC automatically.

The strongest current architecture is:
- CPU native JIT for fresh, small challenges and low-latency responses.
- Static GPU interpreter when many programs must be handled concurrently
  and the kernel is already resident; its relevant metric is independent
  challenges/second and end-to-end first-proof time, not only MH/s.
- Optimized shared-memory CUDA specialization for programs reused across
  enough work to amortize code-generation/compile costs.
- If low-cost direct SASS/codegen is implemented later, remeasure the
  crossover instead of retaining a legacy fixed dispatch threshold.

### Reproducing archived-interpreter adaptations

Fetch the exact historical source (do not modify the hashwx-rs main branch):

~~~bash
git fetch https://github.com/codeandsolder/hashwx-rs.git refs/pull/17/head
git show FETCH_HEAD:experiments/gpu-static-interpreter/static_interpreter.cu \
  > /path/to/build/static_interpreter.cu
~~~

Apply the appropriate standalone patch from bench/patches into that
**build directory**, then compile with nvcc for sm_86:
- static-interpreter-geometry.patch: branchless read and multi-program
  geometry sweep; original 180x4 oracle validation.
- static-interpreter-single.patch: one broadcast ProgramSet, unique nonce
  per active lane, exhaustive output checking and shape timings.
- static-interpreter-single-shared.patch: same single-program test but
  full scratch in CUDA shared memory (negative control).
- static-interpreter-oracle-1024.patch: change the original Rust fixture
  generator to emit 1,024 oracle nonces for the single-program workload.

The patches were dry-applied successfully against the exact original source.
The raw measurements and compiler diagnostics live under logs/.
The same benchmark covers **kernel time only**; a production proof engine
still needs upload, dispatch, result transfer, validation and seed churn
benchmarks before any routing decision.

## Native GPU JIT and per-architecture bootstrap (2026-11-23)

The new Rust [IR-to-PTX emitter](generator/src/ptx.rs) produces complete
seed-specific HashWX programs from the architecture-neutral program IR, across
all 13 op kinds, both phases, conditional loops, SipHash and two-word
unaligned shared-memory reads. It is unit tested across 64 seeds.

The [PTX native runner](run-gpu-jit.sh) uses ptxas to compile SASS, then loads
and validates the binary directly using the CUDA Driver API. On RTX 3080:

| Mode | Fresh native compilation | 65,536-hash throughput |
| --- | ---: | ---: |
| ptxas -O0 | 155 ms | 0.23 MH/s |
| ptxas -O1 | 485 ms | 2.76 MH/s |
| ptxas -O2 | ~600 ms | 3.94 MH/s |
| **ptxas -O3** | **~620 ms** | **~4.10 MH/s** |
| NVRTC CUDA C++ | 1.65–2.40 seconds | up to 5.46 MH/s |

The full O3 source and kernel each returned correct GPU outputs on all
**16 distinct programs x 65,536 distinct nonces: 1,048,576 verified hashes**.
O3 rates by seed ranged roughly 3.7–4.1 MH/s, and CUDA module loading was
usually ~0.2–0.6 ms. The 463-hash kernel typically took ~0.20–0.23 ms.

### Automatically harvested per-SM native shell

The [bootstrap](bootstrap/src/main.rs) queries GPU compute capability and
compiles a reference PTX kernel once per architecture/toolkit/source identity.
It parses CUBIN ELF section headers rather than assuming fixed offsets,
locates native instruction text and exit-offset metadata, extracts up to
eight native SASS instruction samples per mnemonic from nvdisasm, and caches
them in an explicitly supplied build directory. Repeated runs show cache hits.

The [shell generator](bootstrap/build-shell.sh) builds an inert, oversized
native kernel shell with the same HashWX entry ABI and sufficient resource
reservation for an entire generated kernel. Our SM86 shell reserves
**128 registers/thread and 291,840 bytes of native instruction space**.

The Rust patcher validates architecture, code capacity, register capacity,
kernel metadata and exit sites, then inserts a completely different native
instruction stream and relocates exit metadata. It permits one absent/present
explicit-zero stack-size attribute; all other ABI attributes must match.
**The same shell successfully hosted all 16 independently generated SASS
programs**, each verified for 65,536 hashes on the physical RTX 3080.
Throughput remained ~3.8–4.1 MH/s. A native-to-native transplant between
two ordinary shells and an exact byte-for-byte self-transplant also passed.

Offline calibration compiled successfully for **SM89, SM90, and SM120** with
CUDA 13.1, discovering different native ELF text and exit-offset locations.
Only SM86 was run on physical hardware. These calibration samples are **not
yet a complete SASS operand encoder**; their primary value is automating the
per-ISA CUBIN layout, opcode specimen collection, and resource handling.

### Reproduce

Generate 16 seed programs plus independent CPU JIT oracles on Acer:

~~~bash
cargo +nightly-2026-11-22 run --offline --locked --release \
  --manifest-path generator/Cargo.toml \
  --target-dir /home/jan/build/hashwx-jit/generator-target \
  -- /home/jan/build/hashwx-jit/seeds 16 65536
~~~

Transfer the seeds directory to the GPU host. Then:

~~~bash
./run-gpu-jit.sh /path/to/seeds /path/to/build/hashwx-jit 7
~~~

The runner compiles the std-only Rust bootstrap, calibrates the connected
architecture, builds its cached native shell, runs ptxas once per new seed
(CUBIN cached by seed/source/SM/compiler), patches the native SASS into that
shell (separately cached), and validates every GPU output. Repeat runs re-use
the calibration, compiled CUBINs, and patched binaries. All generated
binaries and caches remain in the user-supplied build directory.

### Direct-SASS emitter still needed

This is a fully verified **IR → PTX → ptxas → native CUBIN → patched native
shell** pipeline. Unlike the existing [HashX SM86 native SASS JIT](https://github.com/codeandsolder/challenge-solvers/pull/65),
HashWX still invokes ptxas for each fresh program. The next milestone is
**IR → encoded SASS directly in Rust**, reusing HashX's 64-bit arithmetic
lowerings but extending its encoder with LDS/STS shared-memory instructions,
branch offsets/reconvergence, register allocation, scheduling controls and
per-architecture instruction fields. The proven patcher and automatic shell
remove the CUBIN-format barrier, but do not yet make source-to-SASS JIT
generation microsecond-fast.
