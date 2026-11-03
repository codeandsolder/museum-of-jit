#!/usr/bin/env python3
from __future__ import annotations

import os
import re
import subprocess
import sys


def count(name: str, default: int, allow_zero: bool = False) -> int:
    raw = os.environ.get(name, str(default))
    try:
        value = int(raw)
    except ValueError as exc:
        raise SystemExit(f"{name} has invalid count: {raw}") from exc
    if value < 0 if allow_zero else value <= 0:
        raise SystemExit(f"{name} has invalid count: {raw}")
    return value


def main() -> None:
    binary = os.environ.get("MUSEUM_RPYTHON_HASHWX")
    if not binary:
        raise SystemExit(
            "MUSEUM_RPYTHON_HASHWX must point to the translated RPython executable"
        )
    warmup = count("MUSEUM_WARMUP", 30000, True)
    iterations = count("MUSEUM_ITERATIONS", 300000)
    samples = count("MUSEUM_SAMPLES", 7)
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
