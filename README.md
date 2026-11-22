# Museum of JIT

A reproducible collection of real programs fed through wildly different JITs, compiler backends, and eventually accelerators.

The first exhibit is HashWX: a compact generated integer workload with branches, rotates, multiplication, scratch-memory traffic, per-program specialization, and a native implementation good enough to make compiler mistakes obvious. The repository exists because one-off experiments with RyuJIT, HotSpot, Graal, V8, Wasmtime, and hand-emitted x86 kept becoming useful again.

## Rules of the museum

- **Same workload, honest backend.** An implementation may use the runtime in the most natural way available, but may not change the algorithm.
- **Historical source is immutable.** `archive/` contains recovered experiment source. Normalized benchmark adapters live under `workloads/`.
- **Results are raw data first.** Curated runs record host identity, runtime version, exact implementation revision, warmup, every measured sample, compile/build latency where observable, code size where observable, correctness check, and command provenance.
- **No benchmark artifacts in the tree.** The native Rust harness owns per-run scratch directories and discovers real build artifacts from Cargo's JSON stream. No copied source trees, shell runner wrappers or jq are required.
- **References stay visible.** Hand-written/native Rust and canonical upstream implementations are first-class reference points, not hidden baselines.
- **Runtime abuse gets labelled.** `idiomatic`, `generated-source-or-bytecode`, `direct-ir-or-machine-code`, and `meta-jit-interpreter` are distinct implementation modes.

## Current exhibits

`workloads/hashwx-fixed-v1/` is the recovered fixed HashWX program used by the 2026-10-30 cross-runtime experiments. It currently has normalized adapters for:

- Rust + dynasm direct x86-64 emission
- Cranelift 0.137.1 direct native-IR JIT (fully generated HashWX, not a Wasm host)
- Cranelift 0.137.1 native in-JIT batch loop (separate adapter, no per-hash Rust/JIT boundary)
- LLVM 21.1.8 ORC LLJIT native IR with O0/O1/O2/O3 policies and a generated in-JIT batch function (O0 fastest for this workload)
- .NET / RyuJIT from `DynamicMethod` IL
- .NET / RyuJIT static-source ceiling
- Java / HotSpot straight-line source
- Java / HotSpot split helpers, including the `@ForceInline` experiment
- CPython 3.15 experimental JIT generated Python (**completed negative-result exhibit**; reproducible build retained, performance work stopped)
- PyPy generated Python using `__pypy__.intop`
- V8 and Wasmtime over the same generated fixed-program WebAssembly module
- RPython/PyPy tracing meta-JIT over a fixed bytecode interpreter

The complete recovered C#/Java/Rust scratch source is under `archive/hashwx-fixed-v1/`, including variants that lost. They are evidence, not recommended implementations.

`workloads/hashwx-seeded-v1/` records the pinned canonical HashWX/HashX and `hashwx-rs` references used by the production release gate. Wasm/V8/Wasmtime and HashX are being folded into the same result format rather than pretending they are the exact same fixed-program micro-workload.

`workloads/challenge-sha256-v1/` adds a portable C# `Vector<uint>`
SHA-256 nonce-solving comparison against scalar RyuJIT, .NET's
`SHA256.HashData` library, and the actual pinned Rust production
`challenge-solvers` SIMD backend. See its
[workload definition](workloads/challenge-sha256-v1/README.md) for scope
and benchmark limitations.

## Running

The maintained benchmark orchestrator is the native Linux Rust CLI
`museum-bench`. Build it with the host's existing managed compiler cache,
keeping its own artifacts outside the repository:

```bash
cargo install --locked --path tools/benchmark --root "$HOME/.local"
museum-bench list
museum-bench run --impl rust-dynasm
museum-bench run --output results/my-host.json
museum-bench list --workload challenge-sha256-v1
```

Use `--root /path/to/museum-of-jit` when running outside the checkout.
Host-specific runtime paths, prebuilt binaries, compiler selection and build
CPU sets are explicitly loaded with `--config /path/to/runtime.toml`.
[The example configuration](tools/runtime.example.toml) lists supported
keys; no implicit config discovery or custom environment overrides exist.
Bare executable names use the normal OS PATH; other paths are absolute or
relative to the repository root. Source builds require the configured managed
sccache client and disable automatic daemon startup. Benchmark runtime
versions remain pinned as documented in `tools/runtime-lock.toml`.

Set benchmark dimensions with `--warmup`, `--iterations`, `--samples`
(defaults 30000, 300000, 7), and the whole-run deadline with `--timeout`
(seconds, default 120). Counts use the common signed 32-bit range; the
challenge workload additionally caps counts at 160000000. Warmup may be
zero; iterations and samples must be positive. Every implementation attests
its actually parsed dimensions in exactly one
`BENCHMARK_DIMENSIONS v1 WARMUP ITERATIONS SAMPLES` line. Stale prebuilt
executables must be rebuilt. The CLI captures the exact native benchmark
and preceding tool argv, runtime version, known runtime JIT settings and
artifact SHA-256 where available, in each JSON record. No global
environment mutation or shell expansion is involved. Each run owns a
process group and scratch directory, including timeouts and interrupts;
Linux child-subreaper ownership also reaps abandoned grandchildren.
Failed, invalid and timed-out exhibits remain in the JSON report, and
the CLI exits nonzero when any exhibit fails.

Host provenance includes the process CPU affinity on platforms that expose it, so pinned single-CPU runs cannot be mistaken for unconstrained runs.

The result format is documented by `schema/result.schema.json`. A future Salad/fleet runner can submit exactly the same records; nothing in the schema assumes one machine or one language.

## First Zen 5 snapshot

The first curated cold-storage run on an AMD EPYC 9655 lands in `results/epyc-9655-zen5-2026-10-31.json`: direct Rust/dynasm is 442.4 kH/s median, .NET 10.0.403 dynamic-pointer is 412.1 kH/s (93.1% of direct emission), and static C# is 290.2 kH/s. See `results/README.md` for accepted, rejected, and historical experiments.

The reproducibly built CPython 3.15 tier-2 JIT was investigated and retired as a HashWX performance candidate: it regressed substantially relative to the same runtime with the JIT disabled, and both trailed direct native emission by orders of magnitude. The source, builder, and raw diagnostic measurements remain available; no noncanonical CPython measurement was added to the leaderboard. See [`results/diagnostics/README.md`](results/diagnostics/README.md) for the trace and policy experiments.
