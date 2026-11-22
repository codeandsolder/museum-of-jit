# Seed-specialized Zipp native WebGPU experiment

This is an experimental, separate GPU exhibit rather than a new entry on the
single-thread fixed-program CPU leaderboard. Both algorithms consume the same
read-only generated IR from the museum's pinned `hashwx-rs` revision. Native
Zipp is pinned to `9507aa98bb4dba013eaab4e684b5cbe39e5d593c` (0.0.37).

The stock Zipp GPU Lab graph protocol is float32 and cannot express these
workloads directly. Instead, the harness evaluates the dispatch driver in
Zipp's **trusted** JavaScript runtime via `GpuHost::eval`, using the existing
WebGPU-to-wgpu shim. GPU compilation is the native backend's WGSL pipeline
compiler, not Zipp's CPU JIT. This does not expose shaders to untrusted guests
or change the public GPU Lab graph protocol.

The generated shaders represent each 64-bit word as two u32 limbs and emit
seed-specialized operations and structured branch loops. HashWX allocates an
independent 16,392-byte scratch area per invocation in a storage buffer.
HashX emits its own signed and unsigned multiply-high operations and its
single taken branch. The GPU never substitutes float arithmetic.

Build outside the checkout using the host's configured managed sccache:

```sh
cargo +nightly-2026-11-22 build --locked --release --target-dir /path/to/museum-build
cargo +nightly-2026-11-22 test --locked --target-dir /path/to/museum-test-build
/path/to/museum-build/release/museum-zipp-gpu hashx 65536 7 auto
/path/to/museum-build/release/museum-zipp-gpu hashwx 4096 7 auto
```

The final argument selects `auto`, `vulkan`, `dx12`, or `metal`. A missing
hardware backend is an error rather than a CPU fallback. Native Windows
Direct3D 12 can be invoked from WSL on the GPU host; CUDA visibility in WSL
does not, by itself, establish a hardware Vulkan adapter.

The initial deterministic fixture uses the seed bytes 0..31 and a corpus of
explicit boundary values plus pseudorandom u64 inputs. The first dispatch is
warmup and every output is compared with the portable reference; subsequent
samples time uploads, dispatch, copy and synchronized readback separately from
WGSL/pipeline compilation. The returned wall times are milliseconds. This
pilot does not yet represent the full alternating-order multi-seed release
gate in the parent exhibit. No production performance claim follows from
shader generation alone.

The optional fifth command-line argument is a 64-digit hexadecimal seed.
Reports include native single-thread CPU JIT compilation and per-batch
times on identical inputs. Every timed GPU readback is checked against
the already independently verified warmup result outside the timed interval.
CPU timings include output materialization but not reference comparison.

## RTX 3080 diagnostic (2026-11-22)

Windows 10 build 19045.6456, NVIDIA 591.86, AMD Ryzen 7 3700X.
Native Windows Vulkan, run through WSL interoperability, executable built
from 32984c8. The GPU and matched single-thread CPU JIT were exercised
sequentially; this was not an isolated release-gate benchmark.

| Algorithm | Batch | GPU median | CPU median | GPU throughput | CPU throughput |
| --- | ---: | ---: | ---: | ---: | ---: |
| HashX | 65536 | 0.8508 ms | 5.1197 ms | 77.03 MH/s | 12.80 MH/s |
| HashWX | 4096 | 3.4409 ms | 10.1681 ms | 1.19 MH/s | 0.403 MH/s |

All 557,056 outputs (warmups plus seven samples per workload) matched.
Observed first/repeat pipeline compilation times were 891/16.0 ms for
HashX and 6253/44.8 ms for HashWX; driver caching was not controlled.
Raw observations are in `results/diagnostics/rtx3080-zipp-gpu-*`.

DX12 HashX produced an incorrect output on the same input corpus; DX12
HashWX did not finish its 64-input smoke test within 150 seconds.
Do not use DX12 results as validated performance data.
These fixed-seed, single-thread CPU comparisons do not establish
production suitability or an advantage versus all CPU cores.

## Native compiler comparison

The extra CLI modes share the original seed and salt corpus:
```sh
museum-zipp-gpu direct hashx 65536 15 vulkan
museum-zipp-gpu direct hashwx 4096 15 vulkan
museum-zipp-gpu export /path/to/hashx-fixture hashx 65536 1 vulkan
museum-zipp-gpu export /path/to/hashwx-fixture hashwx 4096 1 vulkan
```
The export writes kernel.cu, kernel.wgsl, inputs.bin and expected.bin
(little-endian u64). CUDA uses exact native arithmetic and the same
generated IR, including signed and unsigned multiplication high for HashX,
and separate global 16,392-byte scratch for each HashWX invocation.
The CUDA kernel is a fresh equivalent fixed-program reference, not the
historical shared-memory AOT layout whose temporary source was deleted.

The standalone CUDA driver harness in tools/cuda_compare.cpp needs a CUDA
toolkit, the driver and NVRTC. In the target WSL environment:
```sh
g++ -std=c++17 -O2 -Wall -Wextra -Wconversion -Werror \
    -I/usr/local/cuda/include tools/cuda_compare.cpp \
    -L/usr/local/cuda/lib64 -Wl,-rpath,/usr/local/cuda/lib64 \
    -L/usr/lib/wsl/lib -Wl,-rpath,/usr/lib/wsl/lib \
    -lcuda -lnvrtc -o /path/to/build/cuda-compare
cd /path/to/hashx-fixture
/usr/local/cuda/bin/nvcc --cubin -O3 -arch=sm_86 -std=c++17 -Xptxas=-v kernel.cu -o kernel.cubin
/path/to/build/cuda-compare . hashx cubin 7 64
/path/to/build/cuda-compare . hashx nvrtc 7 64
/path/to/build/cuda-compare . hashx ptx 7 64
```
Repeat for the HashWX fixture, setting algorithm hashwx. AOT excludes
offline compilation and module loading from timed samples. Runtime NVRTC
reports compile and module load separately (native cubin or driver-compiled
PTX); both also report kernel-event and host-visible batch times, including
upload and readback. The oracle compares every output in every batch.
Direct wgpu uses the same WGSL and analogous synchronized batch timings as
Zipp; CPU timings are excluded. GPU modes differ in host submission/transfer
implementation (native Windows Vulkan vs WSL CUDA), so comparisons between
those APIs are diagnostic, not isolated measurements of GPU instructions.

### RTX 3080 results (2026-11-22)

Same physical RTX 3080, driver 591.86, CUDA 13.1, default seed, 65536
HashX or 4096 HashWX inputs. Zipp/direct wgpu results pool 45 samples
across three alternating-order rounds; CUDA rows use seven samples at
64 threads/block. All returned hashes matched.

| Implementation | HashX MH/s | HashWX MH/s |
| --- | ---: | ---: |
| Zipp, wall | 61.134 | 1.160 |
| Direct wgpu, wall | 200.600 | 1.205 |
| CUDA offline cubin, wall | 168.730 | 1.643 |
| CUDA offline cubin, kernel | 818.872 | 1.894 |
| CUDA NVRTC native cubin, kernel | 782.575 | 1.892 |
| CUDA NVRTC PTX driver JIT, kernel | 510.851 | 1.891 |

Offline compile: 835 ms / 4931 ms. NVRTC native cubin compile:
524.260 / 4276.824 ms, plus load 0.855 / 0.748 ms.
PTX generation: 36.374 / 59.416 ms, plus driver load/JIT
423.258 / 1254.468 ms (driver cache uncontrolled).
Repeated same-seed WGSL compilation median: Zipp 20.767 / 52.524 ms,
direct wgpu 16.917 / 53.085 ms. Initial direct-wgpu observed compilation
907 / 7169 ms; the initial HashWX direct-wgpu batch had a transient
40+ ms execution phase that did not recur in the paired warm runs.
AOT launch geometry 32,64,128,256,512 was also measured; peak HashWX
kernel median was ~1.94 MH/s at 32 threads/block. Do not treat these
measurements as a theoretical performance bound or a multiseed latency test.

The old compact HashWX CUDA static interpreter was recompiled unchanged
for SM86 using its original 1024-seed fixture. 720 CPU-golden checks
passed. Best original sparse-lane shape yielded 0.178515 MH/s
(90 blocks x 2 active lanes x 463 successive hashes per seed; kernel
time 466.8508 ms). This is a different multiseed workload. Our existing
direct-SASS HashX JIT is SM75-specific and cannot run on the SM86 RTX 3080;
historical Turing kernel-only rates ~365–386 MH/s and code generation
34–70 us are not same-hardware comparison values.
Full raw reports: results/diagnostics/rtx3080-zipp-gpu-comparison-2026-11-22.json.
