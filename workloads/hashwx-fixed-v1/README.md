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

### Reproducible CPython 3.15 JIT build

The curated CPython JIT runtime is built out of tree with the pinned CPython 3.15.2 source and LLVM 21.1.8 stencil toolchain. Supply a target-compatible Linux sysroot; all C/C++ and JIT-stencil compilation is forced through the existing sccache service, with daemon autostart disabled:

```sh
tools/build-cpython-jit.sh /path/to/runtime-root /path/to/build-dir /path/to/sysroot
```

The builder enables PGO, full LTO, and `--enable-experimental-jit`, installs under the build directory, verifies that `PYTHON_JIT=1` produces a real tier-2 executor, performs a relocation smoke test from a different prefix, and records the maximum referenced GLIBC symbol version. Set `MUSEUM_CPYTHON_MAX_GLIBC` to make a target ABI ceiling fatal. `MUSEUM_CPYTHON_OBJDIR` can place disposable configure/compiler state on a different filesystem without moving durable logs or the install prefix; it is part of the resume key, and destructive-path guards reject unsafe object-directory choices. The resume key also fingerprints the sysroot package database when present, so changing installed target packages invalidates configure/build reuse. `MUSEUM_SCCACHE_PROBE_PORT` and `MUSEUM_SCCACHE_JIT_PORT` may route tiny configure probes or JIT stencil compilation to an already-running sccache endpoint; the script never starts a daemon. Sccache transport failures are retried only when the client returns its infrastructure-failure status; ordinary compiler failures are never retried. Configure validation scans both the terminal transcript and Autoconf's `config.log`, including on resumed builds, so cache-service failures cannot silently become false platform-feature negatives. OpenSSL discovery is explicitly restricted to the supplied sysroot: if its development headers/libraries are absent, `_ssl` and `_hashlib` are omitted instead of falling back to build-host `/usr`. The generated Makefile is audited for the expected compiler wrappers, GCC PGO flags, full-LTO flags, tier-2/JIT defines, sysroot, LLVM 21 JIT command, sysroot-scoped OpenSSL policy, and absence of bare host-root include/library/rpath flags before compilation begins.
