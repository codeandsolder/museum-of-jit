#!/usr/bin/env python3
"""Cross-check the native Cranelift JIT against the independent fixed Wasm emitter.

Run with `uv run --no-project --python 3.13 python tools/validate-cranelift-direct.py
--binary /path/to/museum-hashwx-cranelift-direct`. Requires Node and the exact
native binary under validation; never compiles or starts a cache daemon.
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
WASM = ROOT / 'workloads/hashwx-fixed-v1/implementations/wasm-fixed/emit_wasm.py'
CORPUS = ROOT / 'workloads/hashwx-fixed-v1/program.json'
ORACLE = ROOT / 'tools/validation/cranelift-wasm-oracle.mjs'
SEED = 0xC3A7_E517
EDGE_SALTS = (
    0, 1, 2, 3, 4, 5, 31, 32, 63, 64, 255, 256, 1023, 4095,
    4096, 16383, 16384, 65535, 123456, 2**64 - 1, 2**63,
    2**63 - 1, 0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA,
)


def generate_salts(count: int, output: Path) -> None:
    rng = random.Random(SEED)
    with output.open('w', encoding='ascii') as stream:
        for salt in itertools.islice(
            itertools.chain(EDGE_SALTS, range(50_000),
                            (rng.getrandbits(64) for _ in itertools.count())), count
        ):
            stream.write(f'{salt:016x}\n')


def verify_lines(expected: Path, actual: Path, count: int) -> str:
    digest = hashlib.sha256()
    with expected.open('rb') as a, actual.open('rb') as b:
        matched = 0
        for i, (want, got) in enumerate(itertools.zip_longest(a, b), 1):
            if want != got:
                raise ValueError(f'differential mismatch at salt line {i}: expected={want!r}, got={got!r}')
            digest.update(want)
            matched += 1
    if matched != count:
        raise ValueError(f'expected {count} hashes, got {matched}')
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--count', type=int, default=100_000)
    parser.add_argument('--node', default='node')
    args = parser.parse_args()
    if args.count < len(EDGE_SALTS):
        parser.error(f'count must be >= {len(EDGE_SALTS)} to cover boundary salts')
    binary = args.binary.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix='museum-cranelift-differential-') as scratch:
        d = Path(scratch)
        salts = d / 'salts.hex'
        module = d / 'hashwx-fixed.wasm'
        wasm_hashes = d / 'wasm.hex'
        jit_hashes = d / 'cranelift.hex'
        generate_salts(args.count, salts)
        subprocess.run([sys.executable, str(WASM), '--output', str(module)], check=True,
                       stdout=subprocess.DEVNULL)
        subprocess.run([args.node, str(ORACLE), str(module), str(salts),
                        str(wasm_hashes)], check=True)
        env = dict(os.environ, MUSEUM_PROGRAM_JSON=str(CORPUS))
        with jit_hashes.open('wb') as out:
            subprocess.run([str(binary), '--hash-salts', str(salts)], check=True,
                           stdout=out, env=env)
        digest = verify_lines(wasm_hashes, jit_hashes, args.count)
        print(f'CRANELIFT_DIFF_OK count={args.count} sha256={digest} '
              f'corpus={CORPUS.name} wasm_bytes={module.stat().st_size}')


if __name__ == '__main__':
    main()
