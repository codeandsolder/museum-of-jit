#!/usr/bin/env python3
"""Emit the canonical hashwx-fixed-v1 workload as a self-contained Wasm module.

The module exports:
  hash(i64) -> i64
  bench(start: i64, count: i64) -> i64

`bench` keeps the iteration loop inside Wasm so engine comparisons measure the
compiled workload rather than host<->Wasm call overhead.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROGRAM = ROOT / "program.json"
K0 = 0x1716151413121110
K1 = 0x1F1E1D1C1B1A1918

I32 = 0x7F
I64 = 0x7E
EMPTY = 0x40

R0, R1, R2, R3, R4, R5, R6, R7, R8 = range(1, 10)
T0, T1, T2, T3 = range(10, 14)
REP, BC, MP = 14, 15, 16
REGS = (R0, R1, R2, R3, R4, R5, R6, R7)


def uleb(value: int) -> bytes:
    if value < 0:
        raise ValueError(value)
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            byte |= 0x80
        out.append(byte)
        if not value:
            return bytes(out)


def sleb(value: int) -> bytes:
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        sign = byte & 0x40
        done = (value == 0 and not sign) or (value == -1 and sign)
        if not done:
            byte |= 0x80
        out.append(byte)
        if done:
            return bytes(out)


def signed64(value: int) -> int:
    value &= (1 << 64) - 1
    return value if value < (1 << 63) else value - (1 << 64)


def vec(items: list[bytes] | tuple[bytes, ...]) -> bytes:
    return uleb(len(items)) + b"".join(items)


def name(text: str) -> bytes:
    raw = text.encode()
    return uleb(len(raw)) + raw


def section(section_id: int, payload: bytes) -> bytes:
    return bytes((section_id,)) + uleb(len(payload)) + payload


class Code:
    def __init__(self) -> None:
        self.data = bytearray()

    def op(self, opcode: int, immediate: bytes = b"") -> None:
        self.data.append(opcode)
        self.data += immediate

    def local_get(self, index: int) -> None:
        self.op(0x20, uleb(index))

    def local_set(self, index: int) -> None:
        self.op(0x21, uleb(index))

    def local_tee(self, index: int) -> None:
        self.op(0x22, uleb(index))

    def i32_const(self, value: int) -> None:
        self.op(0x41, sleb(value))

    def i64_const(self, value: int) -> None:
        self.op(0x42, sleb(signed64(value)))

    def br(self, depth: int) -> None:
        self.op(0x0C, uleb(depth))

    def br_if(self, depth: int) -> None:
        self.op(0x0D, uleb(depth))

    def call(self, index: int) -> None:
        self.op(0x10, uleb(index))

    def block(self) -> None:
        self.op(0x02, bytes((EMPTY,)))

    def loop(self) -> None:
        self.op(0x03, bytes((EMPTY,)))

    def end(self) -> None:
        self.op(0x0B)

    def i64_load(self, align: int = 0, offset: int = 0) -> None:
        self.op(0x29, uleb(align) + uleb(offset))

    def i64_store(self, align: int = 3, offset: int = 0) -> None:
        self.op(0x37, uleb(align) + uleb(offset))


def copy_local(c: Code, dst: int, src: int) -> None:
    c.local_get(src)
    c.local_set(dst)


def xor_const(c: Code, reg: int, value: int) -> None:
    c.local_get(reg)
    c.i64_const(value)
    c.op(0x85)  # i64.xor
    c.local_set(reg)


def sip(c: Code, a: int, b: int, d: int, e: int) -> None:
    # Names mirror the four SipHash state registers; d/e are the third/fourth.
    for dst, left, right in ((a, a, b), (d, d, e)):
        c.local_get(left)
        c.local_get(right)
        c.op(0x7C)
        c.local_set(dst)
    for reg, rot in ((b, 13), (e, 16)):
        c.local_get(reg)
        c.i64_const(rot)
        c.op(0x89)
        c.local_set(reg)
    for dst, left, right in ((b, b, a), (e, e, d)):
        c.local_get(left)
        c.local_get(right)
        c.op(0x85)
        c.local_set(dst)
    c.local_get(a)
    c.i64_const(32)
    c.op(0x89)
    c.local_set(a)
    for dst, left, right in ((d, d, b), (a, a, e)):
        c.local_get(left)
        c.local_get(right)
        c.op(0x7C)
        c.local_set(dst)
    for reg, rot in ((b, 17), (e, 21)):
        c.local_get(reg)
        c.i64_const(rot)
        c.op(0x89)
        c.local_set(reg)
    for dst, left, right in ((b, b, d), (e, e, a)):
        c.local_get(left)
        c.local_get(right)
        c.op(0x85)
        c.local_set(dst)
    c.local_get(d)
    c.i64_const(32)
    c.op(0x89)
    c.local_set(d)


def reg(index: int) -> int:
    return REGS[index] if index < 8 else R8


def load_src(c: Code, index: int) -> None:
    c.local_get(reg(index))
    c.op(0xA7)  # i32.wrap_i64
    c.i32_const(16383)
    c.op(0x71)  # i32.and
    c.i64_load(align=0)


def emit_op(c: Code, ins: dict[str, int], memory: bool) -> None:
    op = ins["op"]
    dst = reg(ins["dst"])
    src = reg(ins["src"])
    imm = ins["imm"]
    if op <= 2:
        c.local_get(dst)
        c.i64_const(imm)
        c.op((0x84, 0x85, 0x7C)[op])  # or, xor, add
        c.local_set(dst)
        c.local_get(dst)
        if memory:
            load_src(c, ins["src"])
        else:
            c.local_get(src)
        c.op(0x7E)  # i64.mul
        c.local_set(dst)
        return
    if op == 3:
        c.local_get(dst)
        c.local_get(R8)
        c.op(0x7E)
        c.i64_const(imm)
        c.op(0x8A)  # i64.rotr
        c.local_set(dst)
        return

    c.local_get(dst)
    c.i64_const(imm)
    if op <= 6:
        c.op(0x8A)  # rotr
    elif op <= 9:
        c.op(0x87)  # shr_s
    else:
        c.op(0x88)  # shr_u
    c.local_set(dst)
    c.local_get(dst)
    if memory:
        load_src(c, ins["src"])
    else:
        c.local_get(src)
    c.op((0x85, 0x7C, 0x7D)[(op - 4) % 3])  # xor, add, sub
    c.local_set(dst)


def snapshot(c: Code) -> None:
    c.local_get(MP)
    c.i32_const(64)
    c.op(0x6B)
    c.local_set(MP)
    for register, offset in zip(REGS, (56, 48, 40, 32, 24, 16, 8, 0), strict=True):
        c.local_get(MP)
        c.local_get(register)
        c.i64_store(align=3, offset=offset)


def emit_program(c: Code, program: dict, memory: bool) -> None:
    c.block()
    c.loop()
    if not memory:
        snapshot(c)
    instructions = program["instructions"]
    for ins in instructions:
        emit_op(c, ins, memory)
    if not program["unconditional"]:
        branch_reg = reg(instructions[0]["dst"])
        c.local_get(branch_reg)
        c.i64_const(32)
        c.op(0x83)  # i64.and
        c.op(0x50)  # i64.eqz
        c.op(0x45)  # i32.eqz
        c.br_if(1)
    c.local_get(BC)
    c.op(0x45)
    c.br_if(1)
    c.local_get(BC)
    c.i32_const(1)
    c.op(0x6B)
    c.local_set(BC)
    c.br(0)
    c.end()
    c.end()


def phase(c: Code, programs: list[dict], memory: bool) -> None:
    c.i32_const(4)
    c.local_set(REP)
    c.loop()
    c.i32_const(32)
    c.local_set(BC)
    for program in programs:
        emit_program(c, program, memory)
    c.local_get(REP)
    c.i32_const(1)
    c.op(0x6B)
    c.local_tee(REP)
    c.br_if(0)
    c.end()


def hash_body(programs: list[dict]) -> bytes:
    c = Code()
    for register, value in (
        (R0, 0x736F6D6570736575 ^ K0),
        (R1, 0x646F72616E646F6D ^ K1),
        (R2, 0x6C7967656E657261 ^ K0),
        (R3, 0x7465646279746573 ^ K1),
    ):
        c.i64_const(value)
        c.local_set(register)
    c.local_get(R3)
    c.local_get(0)
    c.op(0x85)
    c.local_set(R3)
    sip(c, R0, R1, R2, R3)
    c.local_get(R0)
    c.local_get(0)
    c.op(0x85)
    c.local_set(R0)
    xor_const(c, R2, 0xBB)
    for _ in range(3):
        sip(c, R0, R1, R2, R3)
    for dst, src in ((T0, R3), (T1, R2), (T2, R1), (T3, R0)):
        copy_local(c, dst, src)
    for register, value in ((R0, K0), (R1, K1), (R2, K0), (R3, K1)):
        xor_const(c, register, value)
    for _ in range(4):
        sip(c, R0, R1, R2, R3)
    for dst, src in (
        (R4, R3),
        (R5, R2),
        (R6, R1),
        (R7, R0),
        (R0, T0),
        (R1, T1),
        (R2, T2),
        (R3, T3),
    ):
        copy_local(c, dst, src)
    c.local_get(R0)
    c.local_get(R4)
    c.op(0x85)
    c.i64_const(-8)
    c.op(0x83)
    c.i64_const(3)
    c.op(0x84)
    c.local_set(R8)
    c.i32_const(16384)
    c.local_get(R8)
    c.i64_store(align=3)
    c.i32_const(16384)
    c.local_set(MP)
    phase(c, programs, False)
    phase(c, programs, True)
    sip(c, R0, R1, R2, R3)
    sip(c, R4, R5, R6, R7)
    c.local_get(R3)
    c.local_get(R7)
    c.op(0x85)
    c.local_get(R8)
    c.op(0x85)
    c.end()
    return function_body(((13, I64), (3, I32)), bytes(c.data))


def bench_body() -> bytes:
    # params: 0=start, 1=count; locals: 2=sink, 3=index
    c = Code()
    c.i64_const(0)
    c.local_set(2)
    c.local_get(0)
    c.local_set(3)
    c.block()
    c.loop()
    c.local_get(1)
    c.op(0x50)
    c.br_if(1)
    c.local_get(2)
    c.local_get(3)
    c.call(0)
    c.op(0x85)
    c.local_set(2)
    c.local_get(3)
    c.i64_const(1)
    c.op(0x7C)
    c.local_set(3)
    c.local_get(1)
    c.i64_const(1)
    c.op(0x7D)
    c.local_set(1)
    c.br(0)
    c.end()
    c.end()
    c.local_get(2)
    c.end()
    return function_body(((2, I64),), bytes(c.data))


def function_body(locals_: tuple[tuple[int, int], ...], instructions: bytes) -> bytes:
    decls = [uleb(count) + bytes((valtype,)) for count, valtype in locals_]
    payload = vec(decls) + instructions
    return uleb(len(payload)) + payload


def emit_module(programs: list[dict]) -> bytes:
    types = vec(
        [
            bytes((0x60,)) + vec([bytes((I64,))]) + vec([bytes((I64,))]),
            bytes((0x60,)) + vec([bytes((I64,)), bytes((I64,))]) + vec([bytes((I64,))]),
        ]
    )
    functions = vec([uleb(0), uleb(1)])
    memories = vec([bytes((0x01,)) + uleb(1) + uleb(1)])
    exports = vec(
        [
            name("hash") + bytes((0x00,)) + uleb(0),
            name("bench") + bytes((0x00,)) + uleb(1),
        ]
    )
    code = vec([hash_body(programs), bench_body()])
    return (
        b"\0asm\x01\0\0\0"
        + section(1, types)
        + section(3, functions)
        + section(5, memories)
        + section(7, exports)
        + section(10, code)
    )


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", required=True)
    args = ap.parse_args()
    document = json.loads(PROGRAM.read_text())
    if document.get("format") != "museum-of-jit/hashwx-fixed-program/v1":
        raise SystemExit("unexpected fixed-program format")
    programs = document["programs"]
    if len(programs) != 32 or any(len(p["instructions"]) != 8 for p in programs):
        raise SystemExit("expected 32 programs of 8 instructions")
    module = emit_module(programs)
    Path(args.output).write_bytes(module)
    print(f"WASM_BYTES {len(module)}")


if __name__ == "__main__":
    main()
