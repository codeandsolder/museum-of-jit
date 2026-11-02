# HashWX fixed program v1

This is the exact 32-program × 8-instruction stream recovered from the original Rust/dynasm, C# and Java experiments. `program.json` is generated from the recovered Rust source and is the canonical corpus representation for new adapters.

All normalized adapters use:

- correctness probe: `hash(123456)`
- warmup: 30,000 hashes
- measured sample: 300,000 hashes
- steady-state samples: 7

Historical sources under `archive/hashwx-fixed-v1/` intentionally retain their original warmup/sample choices. Active sources under `implementations/` only normalize measurement counts; algorithm code remains the recovered implementation.

The V8 and Wasmtime adapters share one generated WebAssembly module. It exports both `hash(i64) -> i64` for the normal correctness probe and `bench(start, count) -> i64`, which keeps the measured iteration loop inside Wasm so host-language call overhead is not confused with engine throughput. The module is regenerated from `program.json` in a temporary directory for every run; no generated `.wasm` is committed.

The workload is useful for compiler/JIT comparisons because the generated program is fixed, so every backend sees the same control flow and operations. It is not a replacement for the seeded/corpus HashWX benchmark, which tests compile latency and generation variance separately.

The `rpython-jitdriver` adapter is a distinct meta-JIT exhibit rather than a faster spelling of generated Python. A tiny RPython interpreter consumes the same fixed instruction stream with bytecode position as JIT-green state, allowing PyPy's tracing meta-JIT to residualize the workload. Its translated executable is built separately from the exact pinned PyPy/RPython source and supplied through `MUSEUM_RPYTHON_HASHWX`.
