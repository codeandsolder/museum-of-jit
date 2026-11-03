# Museum of JIT

A reproducible collection of real programs fed through wildly different JITs, compiler backends, and eventually accelerators.

The first exhibit is HashWX: a compact generated integer workload with branches, rotates, multiplication, scratch-memory traffic, per-program specialization, and a native implementation good enough to make compiler mistakes obvious. The repository exists because one-off experiments with RyuJIT, HotSpot, Graal, V8, Wasmtime, and hand-emitted x86 kept becoming useful again.

## Rules of the museum

- **Same workload, honest backend.** An implementation may use the runtime in the most natural way available, but may not change the algorithm.
- **Historical source is immutable.** `archive/` contains recovered experiment source. Normalized benchmark adapters live under `workloads/`.
- **Results are raw data first.** Curated runs record host identity, runtime version, exact implementation revision, warmup, every measured sample, compile/build latency where observable, code size where observable, correctness check, and command provenance.
- **No benchmark artifacts in the tree.** Runners build in temporary directories and clean them.
- **References stay visible.** Hand-written/native Rust and canonical upstream implementations are first-class reference points, not hidden baselines.
- **Runtime abuse gets labelled.** `idiomatic`, `generated-source-or-bytecode`, `direct-ir-or-machine-code`, and `meta-jit-interpreter` are distinct implementation modes.

## Current exhibits

`workloads/hashwx-fixed-v1/` is the recovered fixed HashWX program used by the 2026-10-30 cross-runtime experiments. It currently has normalized adapters for:

- Rust + dynasm direct x86-64 emission
- .NET / RyuJIT from `DynamicMethod` IL
- .NET / RyuJIT static-source ceiling
- Java / HotSpot straight-line source
- Java / HotSpot split helpers, including the `@ForceInline` experiment
- CPython JIT generated Python
- PyPy generated Python using `__pypy__.intop`
- V8 and Wasmtime over the same generated fixed-program WebAssembly module
- RPython/PyPy tracing meta-JIT over a fixed bytecode interpreter

The complete recovered C#/Java/Rust scratch source is under `archive/hashwx-fixed-v1/`, including variants that lost. They are evidence, not recommended implementations.

`workloads/hashwx-seeded-v1/` records the pinned canonical HashWX/HashX and `hashwx-rs` references used by the production release gate. Wasm/V8/Wasmtime and HashX are being folded into the same result format rather than pretending they are the exact same fixed-program micro-workload.

## Running

Use `uv`; the orchestration code has no third-party Python dependencies.

```bash
uv run --no-project --python 3 tools/bench.py --list
uv run --no-project --python 3 tools/bench.py --impl rust-dynasm
uv run --no-project --python 3 tools/bench.py --impl all --output results/my-host.json
```

Runtime-specific binaries can be supplied with environment variables documented by each runner. `tools/runtime-lock.toml` records the versions used for curated runs. Benchmark dimensions are controlled uniformly by `MUSEUM_WARMUP`, `MUSEUM_ITERATIONS`, and `MUSEUM_SAMPLES`; the orchestrator validates them and records the exact values in each result. Java runners also accept `MUSEUM_JAVAC_RELEASE` so a newer `javac` can emit bytecode for an older JVM when comparing runtimes.

Host provenance includes the process CPU affinity on platforms that expose it, so pinned single-CPU runs cannot be mistaken for unconstrained runs.

The result format is documented by `schema/result.schema.json`. A future Salad/fleet runner can submit exactly the same records; nothing in the schema assumes one machine or one language.

## First Zen 5 snapshot

The first curated cold-storage run on an AMD EPYC 9655 lands in `results/epyc-9655-zen5-2026-10-31.json`: direct Rust/dynasm is 442.4 kH/s median, .NET 10.0.403 dynamic-pointer is 412.1 kH/s (93.1% of direct emission), and static C# is 290.2 kH/s. See `results/README.md` for rejected/timeout bring-up notes.
