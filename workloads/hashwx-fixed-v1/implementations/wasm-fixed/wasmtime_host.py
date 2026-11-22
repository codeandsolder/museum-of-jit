#!/usr/bin/env python3
from __future__ import annotations

import importlib.metadata
import sys
import time
from pathlib import Path

import wasmtime

EXPECTED = 0xFD5D450B767035C5
MASK = (1 << 64) - 1


def count(raw: str, minimum: int = 1) -> int:
    if not raw or not raw.isascii() or not raw.isdecimal():
        raise SystemExit(f"invalid decimal count: {raw!r}")
    value = int(raw)
    if not minimum <= value <= 2**31 - 1:
        raise SystemExit(f"count out of range: {value}")
    return value


def main() -> None:
    if len(sys.argv) != 5:
        raise SystemExit(
            "usage: wasmtime_host.py module.wasm WARMUP ITERATIONS SAMPLES"
        )
    warmup = count(sys.argv[2], 0)
    iterations = count(sys.argv[3])
    samples = count(sys.argv[4])
    print(f"BENCHMARK_DIMENSIONS v1 {warmup} {iterations} {samples}")
    module_bytes = Path(sys.argv[1]).read_bytes()

    t0 = time.perf_counter_ns()
    config = wasmtime.Config()
    config.parallel_compilation = False
    engine = wasmtime.Engine(config)
    module = wasmtime.Module(engine, module_bytes)
    store = wasmtime.Store(engine)
    instance = wasmtime.Instance(store, module, [])
    build_us = (time.perf_counter_ns() - t0) / 1000.0
    exports = instance.exports(store)
    hash_one = exports["hash"]
    bench = exports["bench"]
    if not isinstance(hash_one, wasmtime.Func) or not isinstance(bench, wasmtime.Func):
        raise TypeError("hash and bench exports must be Wasm functions")

    t0 = time.perf_counter_ns()
    check = int(hash_one(store, 123456)) & MASK
    first_us = (time.perf_counter_ns() - t0) / 1000.0
    if check != EXPECTED:
        raise SystemExit(f"wrong result: {check:016x} != {EXPECTED:016x}")

    bench(store, 0, warmup)
    version = importlib.metadata.version("wasmtime")
    print(f"ENGINE wasmtime VERSION {version}")
    print(
        f"BUILD_US {build_us:.3f} FIRST_US {first_us:.3f} CODE {len(module_bytes)} CHECK {check:016x}"
    )
    for _ in range(samples):
        t0 = time.perf_counter_ns()
        sink = int(bench(store, 0, iterations)) & MASK
        seconds = (time.perf_counter_ns() - t0) / 1e9
        print(f"RATE {iterations / seconds:.3f} sink={sink:016x}")


if __name__ == "__main__":
    main()
