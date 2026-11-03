#!/usr/bin/env python3
import argparse
import json
import os
import platform
import sys
import time
from pathlib import Path

MASK = (1 << 64) - 1
SIGN = 1 << 63
K0 = 1663540288323457296
K1 = 2242261671028070680
EXPECTED_CHECK = 0xFD5D450B767035C5
ROOT = Path(__file__).resolve().parents[2]
PROGRAM_PATH = ROOT / "program.json"


def rotr_expr(x: str, n: int, engine: str) -> str:
    if engine == "pypy":
        return f"(urshift({x},{n})|ilshift({x},{64-n}))"
    return f"((({x}>>{n})|({x}<<{64-n}))&MASK)"


def rotl_expr(x: str, n: int, engine: str) -> str:
    if engine == "pypy":
        return f"(ilshift({x},{n})|urshift({x},{64-n}))"
    return f"((({x}<<{n})|({x}>>{64-n}))&MASK)"


def add_expr(a: str, b: str, engine: str) -> str:
    return f"iadd({a},{b})" if engine == "pypy" else f"(({a}+{b})&MASK)"


def sub_expr(a: str, b: str, engine: str) -> str:
    return f"isub({a},{b})" if engine == "pypy" else f"(({a}-{b})&MASK)"


def mul_expr(a: str, b: str, engine: str) -> str:
    return f"imul({a},{b})" if engine == "pypy" else f"(({a}*{b})&MASK)"


def emit_sip(lines: list[str], names: tuple[str, str, str, str], engine: str, indent: str = "    ") -> None:
    a, b, c, d = names
    lines += [
        f"{indent}{a}={add_expr(a,b,engine)}; {c}={add_expr(c,d,engine)}",
        f"{indent}{b}={rotl_expr(b,13,engine)}; {d}={rotl_expr(d,16,engine)}",
        f"{indent}{b}^={a}; {d}^={c}; {a}={rotl_expr(a,32,engine)}",
        f"{indent}{c}={add_expr(c,b,engine)}; {a}={add_expr(a,d,engine)}",
        f"{indent}{b}={rotl_expr(b,17,engine)}; {d}={rotl_expr(d,21,engine)}",
        f"{indent}{b}^={c}; {d}^={a}; {c}={rotl_expr(c,32,engine)}",
    ]


def emit_load(lines: list[str], src: int, engine: str, indent: str) -> None:
    lines += [
        f"{indent}a=r{src}&16383; wi=a>>3; sh=(a&7)<<3",
        f"{indent}if sh:",
    ]
    if engine == "pypy":
        lines.append(f"{indent}    s=urshift(mem[wi],sh)|ilshift(mem[wi+1],64-sh)")
    else:
        lines.append(f"{indent}    s=((mem[wi]>>sh)|(mem[wi+1]<<(64-sh)))&MASK")
    lines.append(f"{indent}else: s=mem[wi]")


def emit_op(lines: list[str], ins: dict, memory: bool, engine: str, indent: str) -> None:
    op, src, dst, imm = (ins[k] for k in ("op", "src", "dst", "imm"))
    if op != 3:
        if memory:
            emit_load(lines, src, engine, indent)
            s = "s"
        else:
            s = f"r{src}"
    d = f"r{dst}"
    if op == 0:
        lines.append(f"{indent}{d}={mul_expr(f'({d}|{imm})',s,engine)}")
    elif op == 1:
        lines.append(f"{indent}{d}={mul_expr(f'({d}^{imm})',s,engine)}")
    elif op == 2:
        lines.append(f"{indent}{d}={mul_expr(add_expr(d,str(imm),engine),s,engine)}")
    elif op == 3:
        lines.append(f"{indent}{d}={rotr_expr(mul_expr(d,'r8',engine),imm,engine)}; bf={d}")
    elif op == 4:
        lines.append(f"{indent}{d}={rotr_expr(d,imm,engine)}^{s}")
    elif op == 5:
        lines.append(f"{indent}{d}={add_expr(rotr_expr(d,imm,engine),s,engine)}")
    elif op == 6:
        lines.append(f"{indent}{d}={sub_expr(rotr_expr(d,imm,engine),s,engine)}")
    elif op in (7, 8, 9):
        shifted = f"irshift({d},{imm})" if engine == "pypy" else f"(((({d}^SIGN)-SIGN)>>{imm})&MASK)"
        if op == 7:
            lines.append(f"{indent}{d}={shifted}^{s}")
        elif op == 8:
            lines.append(f"{indent}{d}={add_expr(shifted,s,engine)}")
        else:
            lines.append(f"{indent}{d}={sub_expr(shifted,s,engine)}")
    elif op in (10, 11, 12):
        shifted = f"urshift({d},{imm})" if engine == "pypy" else f"({d}>>{imm})"
        if op == 10:
            lines.append(f"{indent}{d}={shifted}^{s}")
        elif op == 11:
            lines.append(f"{indent}{d}={add_expr(shifted,s,engine)}")
        else:
            lines.append(f"{indent}{d}={sub_expr(shifted,s,engine)}")
    else:
        raise ValueError(op)


def count_cpython_executors(hash_one) -> int | None:
    try:
        import _opcode
    except ImportError:
        return None
    get_executor = getattr(_opcode, "get_executor", None)
    if get_executor is None:
        return None
    executor_ids = set()
    code = hash_one.__code__
    for offset in range(0, len(code.co_code), 2):
        try:
            executor = get_executor(code, offset)
        except ValueError:
            continue
        executor_ids.add(id(executor))
    return len(executor_ids)


def generate(programs: list[dict], engine: str) -> str:
    lines = ["def hash_one(salt,mem):"]
    i = "    "
    lines += [
        f"{i}v0=0x736f6d6570736575^K0; v1=0x646f72616e646f6d^K1",
        f"{i}v2=0x6c7967656e657261^K0; v3=0x7465646279746573^K1",
        f"{i}v3^=salt",
    ]
    emit_sip(lines, ("v0", "v1", "v2", "v3"), engine)
    lines += [f"{i}v0^=salt; v2^=0xbb"]
    for _ in range(3):
        emit_sip(lines, ("v0", "v1", "v2", "v3"), engine)
    lines += [f"{i}r0=v3; r1=v2; r2=v1; r3=v0", f"{i}v0^=K0; v1^=K1; v2^=K0; v3^=K1"]
    for _ in range(4):
        emit_sip(lines, ("v0", "v1", "v2", "v3"), engine)
    lines += [
        f"{i}r4=v3; r5=v2; r6=v1; r7=v0",
        f"{i}r8=((r0^r4)&~7)|3",
        f"{i}mem[2048]=r8; sp=2048",
    ]
    for memory in (False, True):
        lines.append(f"{i}for _rep in range(4):")
        lines.append(f"{i}    bc=32")
        for p in programs:
            lines.append(f"{i}    while True:")
            indent = i + "        "
            if not memory:
                lines += [
                    f"{indent}sp-=8",
                    f"{indent}mem[sp+7]=r0; mem[sp+6]=r1; mem[sp+5]=r2; mem[sp+4]=r3",
                    f"{indent}mem[sp+3]=r4; mem[sp+2]=r5; mem[sp+1]=r6; mem[sp]=r7",
                ]
            for ins in p["instructions"]:
                emit_op(lines, ins, memory, engine, indent)
            if p["unconditional"]:
                lines.append(f"{indent}if bc==0: break")
            else:
                lines.append(f"{indent}if bc==0 or (bf&32)!=0: break")
            lines.append(f"{indent}bc-=1")
    emit_sip(lines, ("r0", "r1", "r2", "r3"), engine)
    emit_sip(lines, ("r4", "r5", "r6", "r7"), engine)
    lines.append(f"{i}return r3^r7^r8")
    return "\n".join(lines) + "\n"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--engine", choices=("auto", "cpython", "pypy"), default="auto")
    ap.add_argument("--dump-source")
    args = ap.parse_args()
    engine = args.engine
    if engine == "auto":
        engine = "pypy" if platform.python_implementation() == "PyPy" else "cpython"
    if engine == "pypy":
        try:
            from __pypy__ import intop
        except ImportError as exc:
            raise SystemExit("PyPy intop implementation requires PyPy") from exc
        ns = {
            "K0": K0, "K1": K1,
            "iadd": intop.int_add, "isub": intop.int_sub, "imul": intop.int_mul,
            "ilshift": intop.int_lshift, "irshift": intop.int_rshift, "urshift": intop.uint_rshift,
        }
    else:
        ns = {"K0": K0, "K1": K1, "MASK": MASK, "SIGN": SIGN}
    programs = json.loads(PROGRAM_PATH.read_text())["programs"]
    t0 = time.perf_counter_ns()
    source = generate(programs, engine)
    code = compile(source, "<museum-hashwx-generated>", "exec", optimize=2)
    exec(code, ns)  # noqa: S102 - generated source is the benchmark input.
    build_us = (time.perf_counter_ns() - t0) / 1000.0
    if args.dump_source:
        Path(args.dump_source).write_text(source)
    hash_one = ns["hash_one"]
    mem = [0] * 2050
    t0 = time.perf_counter_ns()
    check = hash_one(123456, mem) & MASK
    first_us = (time.perf_counter_ns() - t0) / 1000.0
    if check != EXPECTED_CHECK:
        raise SystemExit(f"wrong result: {check:016x} != {EXPECTED_CHECK:016x}")
    def count(name: str, default: int, minimum: int) -> int:
        raw = os.environ.get(name, str(default))
        try:
            value = int(raw)
        except ValueError as exc:
            raise SystemExit(f"{name} must be an integer, got {raw!r}") from exc
        if value < minimum:
            raise SystemExit(f"{name} must be >= {minimum}, got {value}")
        return value

    warmup = count("MUSEUM_WARMUP", 30000, 0)
    iterations = count("MUSEUM_ITERATIONS", 300000, 1)
    samples = count("MUSEUM_SAMPLES", 7, 1)
    for n in range(warmup):
        hash_one(n, mem)
    jit = getattr(sys, "_jit", None)
    jit_state = "n/a"
    if jit is not None:
        jit_state = str(bool(jit.is_enabled())).lower()
    print(f"ENGINE {engine} IMPL {platform.python_implementation()} VERSION {platform.python_version()} JIT {jit_state}")
    print(f"BUILD_US {build_us:.3f} SOURCE_BYTES {len(source.encode())} FIRST_US {first_us:.3f} CHECK {check:016x}")
    for _ in range(samples):
        sink = 0
        t0 = time.perf_counter_ns()
        for n in range(iterations):
            sink ^= hash_one(n, mem)
        dt = (time.perf_counter_ns() - t0) / 1e9
        print(f"RATE {iterations/dt:.3f} sink={sink & MASK:016x}")
    if engine == "cpython" and jit_state == "true":
        executor_count = count_cpython_executors(hash_one)
        if executor_count is None:
            print("JIT_EXECUTORS unavailable")
        else:
            print(f"JIT_EXECUTORS {executor_count}")


if __name__ == "__main__":
    main()
