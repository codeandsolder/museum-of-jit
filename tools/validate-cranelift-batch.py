#!/usr/bin/env python3
"""Validate the native in-JIT batch loop against native-host and Wasm batches.

Run using uv with a precompiled binary; this validator never compiles Rust,
starts a cache service or places build/test artifacts inside the repository.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import os
from pathlib import Path
import random
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
WASM = ROOT / "workloads/hashwx-fixed-v1/implementations/wasm-fixed/emit_wasm.py"
CORPUS = ROOT / "workloads/hashwx-fixed-v1/program.json"
ORACLE = ROOT / "tools/validation/cranelift-wasm-batch-oracle.mjs"
SEED = 0xBADC_A5E7
U64_MAX = 2**64 - 1


def generate_cases(random_cases: int, output: Path) -> int:
    rng = random.Random(SEED)
    edge_starts = (0, 1, 2, 123456, 2**63, 2**63 - 1, U64_MAX,
                   U64_MAX - 1, U64_MAX - 15, 0x5555555555555555)
    edge_counts = (0, 1, 2, 3, 17, 31, 32, 63, 64, 127, 255)
    cases = list(itertools.product(edge_starts, edge_counts))
    cases.extend((rng.getrandbits(64), rng.randrange(0, 257))
                 for _ in range(random_cases))
    cases.extend([(0, 4096), (0, 16384), (0, 300000),
                  (U64_MAX - 100, 512), (123456, 8192)])
    with output.open("w", encoding="ascii") as stream:
        for start, count in cases:
            stream.write(f"{start:016x} {count:016x}\n")
    return len(cases)


def compare_outputs(reference: Path, candidate: Path, count: int) -> str:
    sha = hashlib.sha256()
    with reference.open("rb") as ref, candidate.open("rb") as got:
        length = 0
        for idx, (a, b) in enumerate(itertools.zip_longest(ref, got), 1):
            if a != b:
                raise ValueError(f"batch mismatch at case {idx}: Wasm={a!r}, native={b!r}")
            sha.update(a)
            length += 1
        if length != count:
            raise ValueError(f"expected {count} cases, got {length}")
    return sha.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--random-cases", type=int, default=512)
    parser.add_argument("--node", default="node")
    args = parser.parse_args()
    if args.random_cases < 0:
        parser.error("random-cases must be nonnegative")
    binary = args.binary.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="museum-cranelift-batch-") as temp:
        root = Path(temp)
        cases = root / "cases.hex"
        wasm = root / "hashwx-fixed.wasm"
        expected = root / "wasm-batches.hex"
        actual = root / "native-batches.hex"
        count = generate_cases(args.random_cases, cases)
        subprocess.run([sys.executable, str(WASM), "--output", str(wasm)],
                       check=True, stdout=subprocess.DEVNULL)
        subprocess.run([args.node, str(ORACLE), str(wasm), str(cases), str(expected)],
                       check=True)
        with actual.open("wb") as output:
            subprocess.run([str(binary), "--batch-cases", str(cases)],
                           check=True, env=dict(os.environ, MUSEUM_PROGRAM_JSON=str(CORPUS)),
                           stdout=output)
        digest = compare_outputs(expected, actual, count)
        print(f"CRANELIFT_BATCH_DIFF_OK count={count} sha256={digest} "
              f"wasm_bytes={wasm.stat().st_size} "
              "(native JIT batch = native host fold = independent Wasm batch)")


if __name__ == "__main__":
    main()
