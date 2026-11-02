#!/usr/bin/env python3
from __future__ import annotations

import importlib.metadata
import os
import sys
import time
from pathlib import Path

import wasmtime

EXPECTED = 0xFD5D450B767035C5
MASK = (1 << 64) - 1


def count(name: str, fallback: int, allow_zero: bool = False) -> int:
    raw = os.environ.get(name)
    if raw is None:
        return fallback
    try:
        value = int(raw)
    except ValueError as exc:
        raise SystemExit(f"{name} has invalid count: {raw}") from exc
    if value < 0 if allow_zero else value <= 0:
        raise SystemExit(f"{name} has invalid count: {raw}")
    return value


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: wasmtime_host.py module.wasm")
    module_bytes = Path(sys.argv[1]).read_bytes()
    warmup = count("MUSEUM_WARMUP", 30000, True)
    iterations = count("MUSEUM_ITERATIONS", 300000)
    samples = count("MUSEUM_SAMPLES", 7)

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
