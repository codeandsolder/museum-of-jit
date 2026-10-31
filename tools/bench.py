#!/usr/bin/env python3
import argparse
import datetime as dt
import hashlib
import json
import os
import platform
import re
import statistics
import subprocess
import sys
import time
import tomllib
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKLOAD = ROOT / "workloads/hashwx-fixed-v1"
CONFIG = WORKLOAD / "implementations.toml"
EXPECTED_CHECK = "fd5d450b767035c5"


def command_output(argv: list[str], env=None) -> str:
    try:
        p = subprocess.run(argv, cwd=ROOT, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=20, check=False)
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return "unavailable"
    text = p.stdout.strip()
    return text if text else f"exit={p.returncode}"


def runtime_version(impl: str, env: dict[str, str]) -> str:
    if impl == "rust-dynasm":
        return command_output(["rustc", "+1.99.0", "--version"], env)
    if impl.startswith("dotnet-"):
        return command_output([env.get("MUSEUM_DOTNET", "dotnet"), "--version"], env)
    if impl.startswith("java-"):
        return command_output([env.get("MUSEUM_JAVA", "java"), "-version"], env)
    if impl == "cpython-jit":
        return command_output([env.get("MUSEUM_CPYTHON", "python3"), "--version"], env)
    if impl == "pypy-intop":
        return command_output([env.get("MUSEUM_PYPY", "pypy3"), "--version"], env)
    return "unknown"


def host_info() -> dict:
    cpu_model = "unknown"
    family = model_id = None
    flags: list[str] = []
    try:
        text = Path("/proc/cpuinfo").read_text()
        block = text.split("\n\n", 1)[0]
        fields = {}
        for line in block.splitlines():
            if ":" in line:
                k, v = line.split(":", 1)
                fields[k.strip()] = v.strip()
        cpu_model = fields.get("model name", cpu_model)
        family = fields.get("cpu family")
        model_id = fields.get("model")
        flags = fields.get("flags", "").split()
    except OSError:
        pass
    try:
        osr = platform.freedesktop_os_release()
        os_name = osr.get("PRETTY_NAME") or osr.get("NAME") or platform.system()
    except OSError:
        os_name = platform.platform()
    return {
        "hostname": platform.node(),
        "arch": platform.machine(),
        "cpu_model": cpu_model,
        "cpu_family": family,
        "cpu_model_id": model_id,
        "os": os_name,
        "kernel": platform.release(),
        "cpu_flags": flags,
    }


def parse_output(text: str) -> dict:
    checks = re.findall(r"\bCHECK\s+([0-9a-fA-F]{16})\b", text)
    rates = [(float(rate), sink.lower()) for rate, sink in re.findall(r"\bRATE\s+([0-9]+(?:\.[0-9]+)?)\s+sink=([0-9a-fA-F]{16})", text)]
    if not checks:
        raise ValueError("benchmark produced no CHECK line")
    if checks[-1].lower() != EXPECTED_CHECK:
        raise ValueError(f"wrong CHECK {checks[-1]}, expected {EXPECTED_CHECK}")
    if not rates:
        raise ValueError("benchmark produced no RATE lines")
    def number(name: str):
        m = re.search(rf"\b{name}\s+([0-9]+(?:\.[0-9]+)?)", text)
        return float(m.group(1)) if m else None
    def integer(name: str):
        m = re.search(rf"\b{name}\s+(\d+)", text)
        return int(m.group(1)) if m else None
    return {
        "check": checks[-1].lower(),
        "build_us": number("BUILD_US"),
        "first_call_us": number("FIRST_US"),
        "code_bytes": integer("CODE"),
        "samples": [{"hashes_per_second": r, "sink": s} for r, s in rates],
    }


def run_one(impl: str, spec: dict, env: dict[str, str], host: dict, corpus_sha: str) -> dict:
    runner = ROOT / spec["runner"]
    if not runner.is_file():
        raise FileNotFoundError(runner)
    started = time.perf_counter()
    proc = subprocess.run([str(runner)], cwd=ROOT, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    elapsed = time.perf_counter() - started
    if proc.returncode != 0:
        raise RuntimeError(f"{impl} failed with exit {proc.returncode}\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}")
    parsed = parse_output(proc.stdout)
    try:
        revision = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=True).stdout.strip()
    except subprocess.CalledProcessError:
        revision = None
    result = {
        "schema_version": 1,
        "run_id": str(uuid.uuid4()),
        "timestamp_utc": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
        "host": host,
        "workload": {"id": "hashwx-fixed-v1", "corpus_sha256": corpus_sha},
        "implementation": {"id": impl, "mode": spec["mode"], "source": spec["source"], "revision": revision},
        "runtime": {"name": spec["runtime"], "version": runtime_version(impl, env), "command": [str(runner.relative_to(ROOT))]},
        "build_us": parsed["build_us"],
        "first_call_us": parsed["first_call_us"],
        "code_bytes": parsed["code_bytes"],
        "check": parsed["check"],
        "warmup_iterations": int(env.get("MUSEUM_WARMUP", "30000")),
        "sample_iterations": int(env.get("MUSEUM_ITERATIONS", "300000")),
        "samples": parsed["samples"],
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "elapsed_seconds": elapsed,
    }
    rates = [x["hashes_per_second"] for x in result["samples"]]
    print(f"{impl:24s} median={statistics.median(rates):12.3f} H/s samples={len(rates)}", file=sys.stderr)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description="Museum of JIT benchmark orchestrator")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--impl", action="append", default=[])
    parser.add_argument("--output")
    args = parser.parse_args()
    specs = tomllib.loads(CONFIG.read_text())
    if args.list:
        for name, spec in specs.items():
            print(f"{name:24s} {spec['mode']:32s} {spec['runtime']}")
        return
    requested = args.impl or ["all"]
    names = list(specs) if "all" in requested else requested
    unknown = [name for name in names if name not in specs]
    if unknown:
        raise SystemExit(f"unknown implementation(s): {', '.join(unknown)}")
    env = os.environ.copy()
    corpus_sha = hashlib.sha256((WORKLOAD / "program.json").read_bytes()).hexdigest()
    host = host_info()
    results = [run_one(name, specs[name], env, host, corpus_sha) for name in names]
    rendered = json.dumps(results, indent=2) + "\n"
    if args.output:
        out = Path(args.output)
        if not out.is_absolute():
            out = ROOT / out
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(rendered)
        print(out, file=sys.stderr)
    else:
        sys.stdout.write(rendered)


if __name__ == "__main__":
    main()
