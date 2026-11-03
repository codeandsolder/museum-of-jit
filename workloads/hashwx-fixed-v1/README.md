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

Build the translated RPython executable outside the repository tree with the pinned, checksum-verified PyPy 8.0.1 toolchain:

```sh
tools/build-rpython-jitdriver.sh /path/to/runtime-root /path/to/build-dir
```

The translation is forced to one make job, compiles generated C through the host's existing `sccache` client with server auto-start disabled, and rejects an artifact if the translation log reports local sccache fallback. The resulting `hashwx-rpython-jit` is supplied to the benchmark runner as `MUSEUM_RPYTHON_HASHWX`.

### Native Cranelift IR JIT

The `cranelift-direct` adapter parses the canonical `program.json` and emits the complete fixed program as native Cranelift IR, without a bytecode interpreter or WebAssembly frontend. This is a distinct backend from the `wasmtime-wasm` adapter, which compiles the same algorithm after lowering it to the 14,166-byte shared WebAssembly module. Its `hash(u64) -> u64` JIT function runs inside a Rust host measurement loop, whereas Wasmtime's `bench` loop runs inside Wasm. Compare their rates with that boundary difference in mind.

The pinned implementation uses Cranelift **0.137.1** with explicit `opt_level=speed` (bare `JITBuilder` defaults to `none`), Rust **1.99**, and the host's existing sccache service. The runner accepts an already compiled binary through `MUSEUM_CRANELIFT_BIN`; otherwise it performs an isolated out-of-tree Cargo build with `--locked --release`, requires a configured sccache wrapper and explicitly disables sccache server auto-start. The JIT holds its executable-memory module alive while calling its native pointer; `BUILD_US` measures parse + IR generation + JIT compilation + finalization separately from `FIRST_US` and timed sample loops. No native code size is reported without a trustworthy source.

Validate every hash for 100,000 deterministic salts (including boundary values) against the independently generated V8/Wasm oracle before accepting benchmark results:

```sh
uv run --no-project --python 3.13 python tools/validate-cranelift-direct.py \
    --binary /path/to/museum-hashwx-cranelift-direct --count 100000
```

The validator uses a temporary directory, does not compile Rust or start any sccache service, compares every 16-digit output exactly, and reports a SHA-256 digest. The compiled binary's diagnostic `--hash-salts SALTS.hex` mode invokes the same native function as the timed benchmark and prints one hash per provided 16-digit salt. These are optional correctness probes, never substituted for the canonical 30k-warmup/7×300k CPU-pinned benchmark.

### Reproducible CPython 3.15 JIT build

The preserved **experimental** CPython JIT runtime is built out of tree with the pinned CPython 3.15.2 source and LLVM 21.1.8 stencil toolchain. Supply a target-compatible Linux sysroot; all C/C++ and JIT-stencil compilation is forced through the existing sccache service, with daemon autostart disabled:

```sh
tools/build-cpython-jit.sh /path/to/runtime-root /path/to/build-dir /path/to/sysroot
```

The builder enables PGO, full LTO, and `--enable-experimental-jit`, installs under the build directory, verifies that `PYTHON_JIT=1` produces a real tier-2 executor, performs a relocation smoke test from a different prefix, and records the maximum referenced GLIBC symbol version. Set `MUSEUM_CPYTHON_MAX_GLIBC` to make a target ABI ceiling fatal. `MUSEUM_CPYTHON_OBJDIR` can place disposable configure/compiler state on a different filesystem without moving durable logs or the install prefix; it is part of the resume key, and destructive-path guards reject unsafe object-directory choices. The resume key fingerprints the builder script itself, the sysroot package database, libc/header identity, and the target binutils inputs, so changing build policy or target packages invalidates configure/build reuse. The C/C++ and JIT stencil compiler is pinned LLVM 21 Clang, using LLVM InstrProf for PGO. The target sysroot supplies libc, startup objects, GNU assembler/symbol inspection tools and pinned LLVM 21 archivers, and the C/C++ runtime. Full LTO links through the installed LLD 23 using an explicit, hashed build-local linker facade; its exact executable hash and version are part of the build key. No host C/C++ headers or runtime libraries may enter the artifact. `MUSEUM_SCCACHE_PROBE_PORT` and `MUSEUM_SCCACHE_JIT_PORT` may route tiny configure probes or JIT stencil compilation to an already-running sccache endpoint; the script never starts a daemon. Sccache transport failures are retried only when the client returns its infrastructure-failure status; ordinary compiler failures are never retried. Configure validation scans both the terminal transcript and Autoconf's `config.log`, including on resumed builds, so cache-service failures cannot silently become false platform-feature negatives. OpenSSL discovery is explicitly restricted to the supplied sysroot: if its development headers/libraries are absent, `_ssl` and `_hashlib` are omitted instead of falling back to build-host `/usr`. The generated Makefile is audited for the expected compiler/binutils/LLD wrappers, Clang InstrProf flags, full-LTO flags, tier-2/JIT defines, sysroot, LLVM 21 JIT command, sysroot-scoped OpenSSL policy, and absence of bare host-root include/library/rpath flags before compilation begins.

**Experiment status: complete; not a performance candidate.** The accepted Clang 21/LLVM InstrProf PGO/full-LTO build passed the real tier-2-executor smoke, relocation, and target GLIBC checks. On Zen 5, however, the HashWX generated-Python function ran substantially slower with JIT enabled than disabled; increasing the initial side-exit compilation threshold reduced executor growth but did not change that conclusion. Preserve this builder and adapter to reproduce the negative finding; do not assume the existence of a canonical 30k/7×300k CPython leaderboard row. Raw CPU0 traces, exact run dimensions, and comparisons are in [`results/diagnostics/README.md`](../../results/diagnostics/README.md).
