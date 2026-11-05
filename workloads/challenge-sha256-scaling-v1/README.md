# SHA-NI vs portable-SIMD, 1/2/4/8-thread scaling (Rust)

Standalone fixed-work benchmark comparing **real** challenge-solvers SHA-NI
(four independent SHA-256 instruction streams per worker) with the production
portable SIMD backend (AVX2 eight lanes on Zen 2, AVX-512 sixteen on Zen 5).
The solver dependency is pinned to commit
`3776d8fbce45ae4641719d9ae40f84578be7c228`.

Both backends hash one-block `SHA256(prefix[42] || decimal_nonce[9])`,
with an identical 64-bit target `baddcafe12345678`. Worker 0 uses the exact
Museum fixture; workers 1..7 change one prefix byte, ensuring independent
non-overlapping messages while preserving the same instruction-layout
characteristics. Before timing, each backend and every worker is checked
against independently computed RustCrypto SHA-256 **full 256-bit digests**
for the first nonce. All samples require exact attempted-candidate accounting.

Worker threads are affinitized using Linux `sched_setaffinity`, with a barrier
releasing them together after warmup and setup. End-to-end aggregate
hashes/s is calculated from the actual number of attempts and wall-clock
duration until all workers finish. The SHA-NI/SIMD execution order alternates
within each thread count across five repetitions. Threads are pinned to
**physical, non-SMT cores** on the Ryzen laptops.

For an **identical candidate set** per worker, use 80 million attempts on
AVX2 machines: one full 8-lane SIMD nonce-prefix group or two 4-lane SHA-NI
groups. Use 160 million on AVX-512: one SIMD 16-lane group or four SHA-NI
groups. The order differs but the message set per worker is identical.
The SHA-256 prefix, length, mutation alphabet and difficulty predicate are
otherwise equal.

## Reproduction

From museum-of-jit checkout on Acer, use sccache and nightly portable SIMD:

~~~bash
export RUSTC_WRAPPER=/usr/local/bin/sccache
export CARGO_TARGET_DIR=/home/jan/build/museum-of-jit/sha-ni-simd-scaling

cargo +nightly-2026-11-03 build --release --locked \
  --manifest-path workloads/challenge-sha256-scaling-v1/rust/Cargo.toml

$CARGO_TARGET_DIR/release/museum-sha256-thread-scaling \
  --cpus 0,1,2,3,4,5,6,7 --iterations 80000000 \
  --warmup 1000000 --samples 5
~~~

On Asus use `--cpus 0,2,4,6,8,10,12,14` so all 8 workers occupy distinct
physical cores (the adjacent logical CPU IDs share an SMT core). Acer
has eight physical cores without SMT: `0,1,2,3,4,5,6,7`.

Cold-storage currently exposes **4 virtual CPUs** (0–3), despite its host
EPYC 9655 having far more physical cores. Use `--cpus 0,1,2,3,0,1,2,3`
and `--iterations 160000000`. The 8-thread datapoint is therefore
**two competing software threads per virtual CPU**, not 8-core scaling.
The other 1/2/4-thread datapoints are virtual-CPU scaling, not guaranteed
dedicated EPYC physical-core performance.

The same statically linked x86_64-musl executable should run on all three.
Use the validated hash printed in the final results so compilation settings
and machine code are identical. Separate host topology, exact raw sample
results, modes, affinity and medians are stored under `results/`.
