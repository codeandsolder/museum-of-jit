#!/usr/bin/env python3
"""Generate the fixed HashWX workload as a tiny RPython meta-JIT interpreter."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROGRAM = ROOT / "program.json"

TARGET_TEMPLATE = r"""# Generated from workloads/hashwx-fixed-v1/program.json; do not edit.
import sys
import time

from rpython.rlib.jit import JitDriver, promote
from rpython.rlib.rarithmetic import intmask, r_uint

K0 = r_uint(0x1716151413121110)
K1 = r_uint(0x1f1e1d1c1b1a1918)
EXPECTED = r_uint(0xfd5d450b767035c5)
OPS = __OPS__
SRCS = __SRCS__
DSTS = __DSTS__
IMMS = __IMMS__
UNCONDITIONAL = __UNCONDITIONAL__
HEX = "0123456789abcdef"


def get_location(memory, pidx, iidx):
    phase_id = 1 if memory else 0
    return "hashwx phase=%d program=%d instruction=%d" % (phase_id, pidx, iidx)


jitdriver = JitDriver(
    greens=["memory", "pidx", "iidx"],
    reds="auto",
    get_printable_location=get_location,
    name="hashwx-fixed-rpython",
)


def hex64(value):
    chars = ["0"] * 16
    shift = 60
    i = 0
    while i < 16:
        chars[i] = HEX[intmask((value >> shift) & r_uint(15))]
        shift -= 4
        i += 1
    return "".join(chars)


def rotl(value, amount):
    return (value << amount) | (value >> (64 - amount))


def rotr(value, amount):
    return (value >> amount) | (value << (64 - amount))


def sip(regs, a, b, c, d):
    regs[a] = regs[a] + regs[b]
    regs[c] = regs[c] + regs[d]
    regs[b] = rotl(regs[b], 13)
    regs[d] = rotl(regs[d], 16)
    regs[b] = regs[b] ^ regs[a]
    regs[d] = regs[d] ^ regs[c]
    regs[a] = rotl(regs[a], 32)
    regs[c] = regs[c] + regs[b]
    regs[a] = regs[a] + regs[d]
    regs[b] = rotl(regs[b], 17)
    regs[d] = rotl(regs[d], 21)
    regs[b] = regs[b] ^ regs[c]
    regs[d] = regs[d] ^ regs[a]
    regs[c] = rotl(regs[c], 32)


def load(regs, mem, src):
    address = intmask(regs[src] & r_uint(16383))
    word = address >> 3
    shift = (address & 7) << 3
    if shift:
        return (mem[word] >> shift) | (mem[word + 1] << (64 - shift))
    return mem[word]


def execute(regs, mem, op, src, dst, imm, memory, bf):
    if op != 3:
        if memory:
            source = load(regs, mem, src)
        else:
            source = regs[src]
    else:
        source = r_uint(0)
    if op == 0:
        regs[dst] = (regs[dst] | r_uint(imm)) * source
    elif op == 1:
        regs[dst] = (regs[dst] ^ r_uint(imm)) * source
    elif op == 2:
        regs[dst] = (regs[dst] + r_uint(imm)) * source
    elif op == 3:
        regs[dst] = rotr(regs[dst] * regs[8], imm)
        bf = intmask(regs[dst])
    elif op == 4:
        regs[dst] = rotr(regs[dst], imm) ^ source
    elif op == 5:
        regs[dst] = rotr(regs[dst], imm) + source
    elif op == 6:
        regs[dst] = rotr(regs[dst], imm) - source
    elif op == 7:
        regs[dst] = r_uint(intmask(regs[dst]) >> imm) ^ source
    elif op == 8:
        regs[dst] = r_uint(intmask(regs[dst]) >> imm) + source
    elif op == 9:
        regs[dst] = r_uint(intmask(regs[dst]) >> imm) - source
    elif op == 10:
        regs[dst] = (regs[dst] >> imm) ^ source
    elif op == 11:
        regs[dst] = (regs[dst] >> imm) + source
    else:
        regs[dst] = (regs[dst] >> imm) - source
    return bf


execute._always_inline_ = True
load._always_inline_ = True
sip._always_inline_ = True
rotl._always_inline_ = True
rotr._always_inline_ = True


def snapshot(regs, mem, sp):
    sp -= 8
    mem[sp + 7] = regs[0]
    mem[sp + 6] = regs[1]
    mem[sp + 5] = regs[2]
    mem[sp + 4] = regs[3]
    mem[sp + 3] = regs[4]
    mem[sp + 2] = regs[5]
    mem[sp + 1] = regs[6]
    mem[sp] = regs[7]
    return sp


snapshot._always_inline_ = True


def phase(regs, mem, memory):
    sp = 2048
    rep = 0
    while rep < 4:
        bc = 32
        pidx = 0
        while pidx < 32:
            unconditional = promote(UNCONDITIONAL[pidx])
            while True:
                if not memory:
                    sp = snapshot(regs, mem, sp)
                bf = 0
                iidx = 0
                while iidx < 8:
                    jitdriver.jit_merge_point(memory=memory, pidx=pidx, iidx=iidx)
                    pc = pidx * 8 + iidx
                    op = promote(OPS[pc])
                    src = promote(SRCS[pc])
                    dst = promote(DSTS[pc])
                    imm = promote(IMMS[pc])
                    bf = execute(regs, mem, op, src, dst, imm, memory, bf)
                    iidx += 1
                if bc == 0:
                    break
                if not unconditional and (bf & 32) != 0:
                    break
                bc -= 1
            pidx += 1
        rep += 1


def hash_one(salt, mem):
    regs = [r_uint(0)] * 9
    regs[0] = r_uint(0x736f6d6570736575) ^ K0
    regs[1] = r_uint(0x646f72616e646f6d) ^ K1
    regs[2] = r_uint(0x6c7967656e657261) ^ K0
    regs[3] = r_uint(0x7465646279746573) ^ K1
    regs[3] = regs[3] ^ salt
    sip(regs, 0, 1, 2, 3)
    regs[0] = regs[0] ^ salt
    regs[2] = regs[2] ^ r_uint(0xbb)
    sip(regs, 0, 1, 2, 3)
    sip(regs, 0, 1, 2, 3)
    sip(regs, 0, 1, 2, 3)
    t0 = regs[3]
    t1 = regs[2]
    t2 = regs[1]
    t3 = regs[0]
    regs[0] = regs[0] ^ K0
    regs[1] = regs[1] ^ K1
    regs[2] = regs[2] ^ K0
    regs[3] = regs[3] ^ K1
    sip(regs, 0, 1, 2, 3)
    sip(regs, 0, 1, 2, 3)
    sip(regs, 0, 1, 2, 3)
    sip(regs, 0, 1, 2, 3)
    regs[4] = regs[3]
    regs[5] = regs[2]
    regs[6] = regs[1]
    regs[7] = regs[0]
    regs[0] = t0
    regs[1] = t1
    regs[2] = t2
    regs[3] = t3
    regs[8] = ((regs[0] ^ regs[4]) & r_uint(-8)) | r_uint(3)
    mem[2048] = regs[8]
    phase(regs, mem, False)
    phase(regs, mem, True)
    sip(regs, 0, 1, 2, 3)
    sip(regs, 4, 5, 6, 7)
    return regs[3] ^ regs[7] ^ regs[8]


def parse_count(argv, index, default, allow_zero):
    if len(argv) <= index:
        return default
    try:
        value = int(argv[index])
    except ValueError:
        return -1
    if allow_zero:
        if value < 0:
            return -1
    elif value <= 0:
        return -1
    return value


def entry_point(argv):
    if len(argv) > 1 and argv[1] == "--version":
        print "RPython JitDriver / PyPy RPython 8.0.1"
        return 0
    warmup = parse_count(argv, 1, 30000, True)
    iterations = parse_count(argv, 2, 300000, False)
    samples = parse_count(argv, 3, 7, False)
    if warmup < 0 or iterations < 1 or samples < 1:
        print "invalid benchmark counts"
        return 2
    mem = [r_uint(0)] * 2050
    started = time.time()
    check = hash_one(r_uint(123456), mem)
    first_ns = int((time.time() - started) * 1000000000.0)
    print "CHECK " + hex64(check)
    print "FIRST_NS " + str(first_ns)
    if check != EXPECTED:
        return 3
    i = 0
    while i < warmup:
        hash_one(r_uint(i), mem)
        i += 1
    sample = 0
    while sample < samples:
        sink = r_uint(0)
        i = 0
        started = time.time()
        while i < iterations:
            sink = sink ^ hash_one(r_uint(i), mem)
            i += 1
        elapsed_ns = int((time.time() - started) * 1000000000.0)
        print "SAMPLE_NS " + str(elapsed_ns) + " SINK " + hex64(sink)
        sample += 1
    return 0


def target(*args):
    return entry_point, None


if __name__ == "__main__":
    sys.exit(entry_point(sys.argv))
"""


def image_literals(document: dict) -> dict[str, str]:
    ops: list[int] = []
    srcs: list[int] = []
    dsts: list[int] = []
    imms: list[int] = []
    unconditional: list[int] = []
    for program in document["programs"]:
        unconditional.append(int(bool(program["unconditional"])))
        for ins in program["instructions"]:
            ops.append(ins["op"])
            srcs.append(ins["src"])
            dsts.append(ins["dst"])
            imms.append(ins["imm"])
    return {
        "__OPS__": repr(ops),
        "__SRCS__": repr(srcs),
        "__DSTS__": repr(dsts),
        "__IMMS__": repr(imms),
        "__UNCONDITIONAL__": repr(unconditional),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", required=True)
    args = ap.parse_args()
    document = json.loads(PROGRAM.read_text())
    if document.get("format") != "museum-of-jit/hashwx-fixed-program/v1":
        raise SystemExit("unexpected program format")
    if len(document["programs"]) != 32:
        raise SystemExit("expected 32 programs")
    rendered = TARGET_TEMPLATE
    for marker, literal in image_literals(document).items():
        rendered = rendered.replace(marker, literal)
    Path(args.output).write_text(rendered)


if __name__ == "__main__":
    main()
