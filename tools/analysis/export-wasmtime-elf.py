#!/usr/bin/env python3
"""Export Wasmtime's actual compiled native module ELF for disassembly.

Run on the *same target CPU* as the benchmark, using the pinned Wasmtime
Python runtime. Unlike disassembling Wasm bytecode, Module.serialize() returns
an ELF containing the generated native function symbols and code. There is no
compilation cache/remote service and no benchmark execution in this tool.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
from pathlib import Path

import wasmtime


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wasm", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--expected-version", default="50.0.1")
    args = parser.parse_args()
    version = importlib.metadata.version("wasmtime")
    if version != args.expected_version:
        parser.error(f"Wasmtime version mismatch: expected {args.expected_version}, got {version}")
    wasm = args.wasm.read_bytes()
    config = wasmtime.Config()
    # Match the canonical Wasmtime benchmark host's compilation policy.
    config.parallel_compilation = False
    engine = wasmtime.Engine(config)
    module = wasmtime.Module(engine, wasm)
    elf = module.serialize()
    if not elf.startswith(b"\x7fELF"):
        parser.error("Wasmtime serialize() result was not an ELF object")
    # Do not silently overwrite a previously captured benchmark/codegen artifact.
    with args.output.open("xb") as output:
        output.write(elf)
    print(
        f"WASMTIME_ELF version={version} bytes={len(elf)} "
        f"wasm_sha256={hashlib.sha256(wasm).hexdigest()} "
        f"elf_sha256={hashlib.sha256(elf).hexdigest()}"
    )


if __name__ == "__main__":
    main()
