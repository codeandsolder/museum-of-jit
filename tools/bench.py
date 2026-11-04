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

BENCH_DEFAULTS = {
    "MUSEUM_WARMUP": (30000, 0),
    "MUSEUM_ITERATIONS": (300000, 1),
    "MUSEUM_SAMPLES": (7, 1),
}


def normalize_benchmark_env(env: dict[str, str]) -> None:
    for name, (default, minimum) in BENCH_DEFAULTS.items():
        raw = env.get(name, str(default))
        try:
            value = int(raw)
        except ValueError as exc:
            raise SystemExit(f"{name} must be an integer, got {raw!r}") from exc
        if value < minimum:
            raise SystemExit(f"{name} must be >= {minimum}, got {value}")
        env[name] = str(value)


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
    if impl == "llvm-orc":
        binary = env.get("MUSEUM_LLVM_ORC_BIN")
        return command_output([binary, "--version"], env) if binary else "LLVM ORC 23.1.4 (build in runner)"
    if impl in ("cranelift-direct", "cranelift-batch"):
        binary = env.get("MUSEUM_CRANELIFT_BIN")
        return command_output([binary, "--version"], env) if binary else "Cranelift 0.137.1 (build in runner)"
    if impl == "cpython-jit":
        return command_output([env.get("MUSEUM_CPYTHON", "python3"), "--version"], env)
    if impl == "pypy-intop":
        return command_output([env.get("MUSEUM_PYPY", "pypy3"), "--version"], env)
    if impl == "v8-wasm":
        return command_output([
            env.get("MUSEUM_NODE", "node"),
            "-p",
            "process.version + ' / V8 ' + process.versions.v8",
        ], env)
    if impl == "wasmtime-wasm":
        return command_output([
            env.get("MUSEUM_WASMTIME_PY", "python3"),
            "-c",
            "import importlib.metadata; print('wasmtime ' + importlib.metadata.version('wasmtime'))",
        ], env)
    if impl == "rpython-jitdriver":
        binary = env.get("MUSEUM_RPYTHON_HASHWX")
        if not binary:
            return "unavailable"
        return command_output([binary, "--version"], env)
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
    try:
        cpu_affinity = sorted(os.sched_getaffinity(0))
    except (AttributeError, OSError):
        cpu_affinity = None
    return {
        "hostname": platform.node(),
        "arch": platform.machine(),
        "cpu_model": cpu_model,
        "cpu_family": family,
        "cpu_model_id": model_id,
        "cpu_affinity": cpu_affinity,
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


def implementation_evidence(impl: str, text: str) -> dict:
    if impl == "llvm-orc":
        if not re.search(r"\bENGINE\s+llvm-orc\b", text) or not re.search(r"\bMODE\s+jit-batch\b", text):
            raise ValueError("LLVM ORC adapter did not attest to the native JIT-internal batch mode")
        return {"jit_batch_internal": True, "llvm_ir_optimization": "O2", "llvm_codegen": "Aggressive"}
    if impl == "cranelift-batch":
        if not re.search(r"\bMODE\s+jit-batch\b", text):
            raise ValueError("Cranelift batch adapter did not execute the JIT-local batch loop")
        return {"jit_batch_internal": True}
    if impl != "cpython-jit":
        return {}

    state = re.search(r"\bJIT\s+(true|false|n/a|unknown)\b", text)
    if state is None:
        raise ValueError("CPython JIT benchmark produced no JIT state")
    if state.group(1) != "true":
        raise ValueError(f"CPython JIT was not enabled (JIT {state.group(1)})")

    executors = re.search(r"\bJIT_EXECUTORS\s+(\d+)\b", text)
    if executors is None:
        raise ValueError("CPython JIT benchmark produced no executor count")
    executor_count = int(executors.group(1))
    if executor_count < 1:
        raise ValueError("CPython JIT benchmark installed no tier-2 executors")
    return {"jit_enabled": True, "jit_executors": executor_count}


def run_one(impl: str, spec: dict, env: dict[str, str], host: dict, corpus_sha: str) -> dict:
    runner = ROOT / spec["runner"]
    if not runner.is_file():
        raise FileNotFoundError(runner)
    try:
        revision = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=True).stdout.strip()
    except subprocess.CalledProcessError:
        revision = None
    base = {
        "schema_version": 1,
        "run_id": str(uuid.uuid4()),
        "timestamp_utc": dt.datetime.now(dt.UTC).isoformat().replace("+00:00", "Z"),
        "host": host,
        "workload": {"id": "hashwx-fixed-v1", "corpus_sha256": corpus_sha},
        "implementation": {"id": impl, "mode": spec["mode"], "source": spec["source"], "revision": revision},
        "runtime": {"name": spec["runtime"], "version": runtime_version(impl, env), "command": [str(runner.relative_to(ROOT))]},
        "warmup_iterations": int(env["MUSEUM_WARMUP"]),
        "sample_iterations": int(env["MUSEUM_ITERATIONS"]),
        "sample_count": int(env["MUSEUM_SAMPLES"]),
        "samples": [],
    }
    timeout = float(env.get("MUSEUM_RUN_TIMEOUT", "120"))
    started = time.perf_counter()
    try:
        proc = subprocess.run([str(runner)], cwd=ROOT, env=env, text=True, capture_output=True, check=False, timeout=timeout)
    except subprocess.TimeoutExpired as exc:
        elapsed = time.perf_counter() - started
        stdout = exc.stdout or ""
        stderr = exc.stderr or ""
        if isinstance(stdout, bytes):
            stdout = stdout.decode(errors="replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode(errors="replace")
        result = base | {
            "status": "timeout", "exit_code": None, "error": f"runner exceeded {timeout:g} seconds",
            "build_us": None, "first_call_us": None, "code_bytes": None, "check": None,
            "stdout": stdout, "stderr": stderr, "elapsed_seconds": elapsed,
        }
        print(f"{impl:24s} TIMEOUT after {elapsed:.1f}s", file=sys.stderr)
        return result
    elapsed = time.perf_counter() - started
    if proc.returncode != 0:
        result = base | {
            "status": "failed", "exit_code": proc.returncode, "error": f"runner exited with status {proc.returncode}",
            "build_us": None, "first_call_us": None, "code_bytes": None, "check": None,
            "stdout": proc.stdout, "stderr": proc.stderr, "elapsed_seconds": elapsed,
        }
        print(f"{impl:24s} FAILED exit={proc.returncode}", file=sys.stderr)
        return result
    try:
        parsed = parse_output(proc.stdout)
        evidence = implementation_evidence(impl, proc.stdout)
    except ValueError as exc:
        result = base | {
            "status": "invalid", "exit_code": proc.returncode, "error": str(exc),
            "build_us": None, "first_call_us": None, "code_bytes": None, "check": None,
            "stdout": proc.stdout, "stderr": proc.stderr, "elapsed_seconds": elapsed,
        }
        print(f"{impl:24s} INVALID {exc}", file=sys.stderr)
        return result
    result = base | {
        "status": "success", "exit_code": proc.returncode, "error": None,
        "build_us": parsed["build_us"], "first_call_us": parsed["first_call_us"],
        "code_bytes": parsed["code_bytes"], "check": parsed["check"], "samples": parsed["samples"],
        "stdout": proc.stdout, "stderr": proc.stderr, "elapsed_seconds": elapsed,
    }
    result["runtime"].update(evidence)
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
    normalize_benchmark_env(env)
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
