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
