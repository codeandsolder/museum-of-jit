#!/usr/bin/env python3
from __future__ import annotations

import os
import re
import subprocess
import sys


def count(raw: str, minimum: int = 1) -> int:
    if not raw or not raw.isascii() or not raw.isdecimal():
        raise SystemExit(f"invalid decimal count: {raw!r}")
    value = int(raw)
    if not minimum <= value <= 2**31 - 1:
        raise SystemExit(f"count out of range: {value}")
    return value


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: host.py WARMUP ITERATIONS SAMPLES")
    binary = os.environ.get("MUSEUM_RPYTHON_HASHWX")
    if not binary:
        raise SystemExit(
            "MUSEUM_RPYTHON_HASHWX must point to the translated RPython executable"
        )
    warmup = count(sys.argv[1], 0)
    iterations = count(sys.argv[2])
    samples = count(sys.argv[3])
    print(f"BENCHMARK_DIMENSIONS v1 {warmup} {iterations} {samples}")
    proc = subprocess.run(
        [binary, str(warmup), str(iterations), str(samples)],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode:
        sys.stdout.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        raise SystemExit(proc.returncode)
    check = re.search(r"^CHECK ([0-9a-f]{16})$", proc.stdout, re.MULTILINE)
    first = re.search(r"^FIRST_NS (\d+)$", proc.stdout, re.MULTILINE)
    measured = re.findall(
        r"^SAMPLE_NS (\d+) SINK ([0-9a-f]{16})$", proc.stdout, re.MULTILINE
    )
    if not check or not first or len(measured) != samples:
        raise SystemExit(f"unexpected RPython output:\n{proc.stdout}")
    print(f"FIRST_US {int(first.group(1)) / 1000.0:.3f} CHECK {check.group(1)}")
    for elapsed_ns, sink in measured:
        seconds = int(elapsed_ns) / 1e9
        print(f"RATE {iterations / seconds:.3f} sink={sink}")


if __name__ == "__main__":
    main()
