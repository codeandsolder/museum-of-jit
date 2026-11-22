use dynasmrt::{DynasmApi, DynasmLabelApi, dynasm, x64::Rq};
use std::time::Instant;

#[derive(Clone, Copy)]
enum Opcode {
    OrMultiply,
    XorMultiply,
    AddMultiply,
    MultiplyRotate,
    RotateXor,
    RotateAdd,
    RotateSub,
    ArithmeticXor,
    ArithmeticAdd,
    ArithmeticSub,
    LogicalXor,
    LogicalAdd,
    LogicalSub,
}

#[derive(Clone, Copy)]
struct Instruction {
    op: Opcode,
    src: Rq,
    dst: Rq,
    imm: i8,
}

impl Instruction {
    const fn new(op: Opcode, src: u8, dst: u8, imm: i8) -> Self {
        Self {
            op,
            src: register(src),
            dst: register(dst),
            imm,
        }
    }
}

struct Program {
    instructions: [Instruction; 8],
    unconditional: bool,
}

// Exact fixed-program fixture, kept in the native emitter so parsing is not
// part of the timed assembly step. Its 256 instructions were checked against program.json.
const PROGRAMS: [Program; 32] = {
    [
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 7, 40),
                Instruction::new(Opcode::LogicalSub, 4, 6, 2),
                Instruction::new(Opcode::RotateSub, 6, 5, 54),
                Instruction::new(Opcode::LogicalAdd, 5, 3, 2),
                Instruction::new(Opcode::XorMultiply, 3, 2, 33),
                Instruction::new(Opcode::XorMultiply, 7, 0, 9),
                Instruction::new(Opcode::OrMultiply, 2, 1, 9),
                Instruction::new(Opcode::RotateAdd, 1, 4, 60),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 4, 20),
                Instruction::new(Opcode::LogicalSub, 3, 5, 2),
                Instruction::new(Opcode::LogicalAdd, 5, 3, 3),
                Instruction::new(Opcode::XorMultiply, 2, 7, 9),
                Instruction::new(Opcode::XorMultiply, 7, 0, 1),
                Instruction::new(Opcode::RotateXor, 4, 2, 45),
                Instruction::new(Opcode::XorMultiply, 6, 1, 1),
                Instruction::new(Opcode::RotateSub, 1, 6, 51),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 0, 10),
                Instruction::new(Opcode::ArithmeticAdd, 7, 2, 1),
                Instruction::new(Opcode::XorMultiply, 4, 6, 33),
                Instruction::new(Opcode::LogicalXor, 3, 5, 1),
                Instruction::new(Opcode::OrMultiply, 0, 1, 33),
                Instruction::new(Opcode::XorMultiply, 6, 4, 9),
                Instruction::new(Opcode::RotateXor, 1, 3, 24),
                Instruction::new(Opcode::ArithmeticSub, 5, 7, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 5, 24),
                Instruction::new(Opcode::ArithmeticAdd, 0, 1, 3),
                Instruction::new(Opcode::LogicalXor, 7, 6, 1),
                Instruction::new(Opcode::LogicalAdd, 4, 3, 3),
                Instruction::new(Opcode::RotateXor, 5, 2, 37),
                Instruction::new(Opcode::XorMultiply, 3, 4, 33),
                Instruction::new(Opcode::AddMultiply, 2, 7, 9),
                Instruction::new(Opcode::AddMultiply, 1, 0, 1),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 1, 49),
                Instruction::new(Opcode::LogicalXor, 1, 0, 2),
                Instruction::new(Opcode::LogicalXor, 6, 4, 1),
                Instruction::new(Opcode::RotateAdd, 7, 5, 10),
                Instruction::new(Opcode::ArithmeticSub, 5, 7, 2),
                Instruction::new(Opcode::OrMultiply, 3, 6, 9),
                Instruction::new(Opcode::OrMultiply, 0, 3, 1),
                Instruction::new(Opcode::OrMultiply, 4, 2, 9),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 3, 63),
                Instruction::new(Opcode::XorMultiply, 0, 5, 1),
                Instruction::new(Opcode::ArithmeticSub, 4, 1, 3),
                Instruction::new(Opcode::LogicalAdd, 1, 4, 2),
                Instruction::new(Opcode::LogicalSub, 3, 2, 2),
                Instruction::new(Opcode::OrMultiply, 6, 7, 9),
                Instruction::new(Opcode::XorMultiply, 7, 6, 33),
                Instruction::new(Opcode::LogicalXor, 2, 0, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 4, 59),
                Instruction::new(Opcode::OrMultiply, 7, 3, 1),
                Instruction::new(Opcode::LogicalXor, 3, 6, 2),
                Instruction::new(Opcode::AddMultiply, 6, 0, 9),
                Instruction::new(Opcode::RotateXor, 0, 2, 11),
                Instruction::new(Opcode::ArithmeticSub, 2, 5, 1),
                Instruction::new(Opcode::XorMultiply, 1, 7, 9),
                Instruction::new(Opcode::RotateSub, 5, 1, 5),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 0, 57),
                Instruction::new(Opcode::ArithmeticAdd, 7, 4, 3),
                Instruction::new(Opcode::LogicalSub, 4, 2, 2),
                Instruction::new(Opcode::AddMultiply, 1, 6, 1),
                Instruction::new(Opcode::LogicalAdd, 5, 3, 2),
                Instruction::new(Opcode::RotateXor, 6, 5, 6),
                Instruction::new(Opcode::OrMultiply, 3, 7, 1),
                Instruction::new(Opcode::XorMultiply, 0, 1, 1),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 6, 35),
                Instruction::new(Opcode::RotateAdd, 1, 2, 18),
                Instruction::new(Opcode::AddMultiply, 6, 7, 1),
                Instruction::new(Opcode::AddMultiply, 5, 3, 33),
                Instruction::new(Opcode::ArithmeticAdd, 4, 0, 3),
                Instruction::new(Opcode::LogicalSub, 3, 1, 3),
                Instruction::new(Opcode::OrMultiply, 0, 5, 9),
                Instruction::new(Opcode::ArithmeticXor, 2, 4, 3),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 4, 34),
                Instruction::new(Opcode::RotateAdd, 3, 1, 16),
                Instruction::new(Opcode::ArithmeticXor, 7, 6, 1),
                Instruction::new(Opcode::XorMultiply, 1, 7, 33),
                Instruction::new(Opcode::XorMultiply, 0, 2, 33),
                Instruction::new(Opcode::RotateAdd, 2, 0, 32),
                Instruction::new(Opcode::ArithmeticAdd, 6, 3, 1),
                Instruction::new(Opcode::XorMultiply, 4, 5, 9),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 1, 1),
                Instruction::new(Opcode::LogicalXor, 6, 7, 1),
                Instruction::new(Opcode::XorMultiply, 3, 2, 1),
                Instruction::new(Opcode::OrMultiply, 7, 3, 1),
                Instruction::new(Opcode::XorMultiply, 4, 5, 33),
                Instruction::new(Opcode::LogicalSub, 5, 4, 1),
                Instruction::new(Opcode::RotateXor, 1, 0, 9),
                Instruction::new(Opcode::ArithmeticXor, 2, 6, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 3, 17),
                Instruction::new(Opcode::RotateAdd, 3, 0, 35),
                Instruction::new(Opcode::LogicalXor, 1, 5, 1),
                Instruction::new(Opcode::OrMultiply, 7, 4, 33),
                Instruction::new(Opcode::RotateSub, 4, 1, 7),
                Instruction::new(Opcode::OrMultiply, 6, 2, 9),
                Instruction::new(Opcode::XorMultiply, 5, 7, 33),
                Instruction::new(Opcode::RotateXor, 0, 6, 55),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 0, 21),
                Instruction::new(Opcode::ArithmeticXor, 6, 5, 3),
                Instruction::new(Opcode::LogicalAdd, 5, 2, 2),
                Instruction::new(Opcode::OrMultiply, 2, 7, 9),
                Instruction::new(Opcode::ArithmeticXor, 7, 3, 1),
                Instruction::new(Opcode::OrMultiply, 3, 1, 1),
                Instruction::new(Opcode::OrMultiply, 0, 6, 1),
                Instruction::new(Opcode::ArithmeticXor, 1, 4, 1),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 5, 41),
                Instruction::new(Opcode::RotateAdd, 3, 6, 55),
                Instruction::new(Opcode::LogicalXor, 6, 7, 1),
                Instruction::new(Opcode::XorMultiply, 2, 4, 9),
                Instruction::new(Opcode::RotateSub, 0, 1, 36),
                Instruction::new(Opcode::LogicalXor, 4, 2, 2),
                Instruction::new(Opcode::OrMultiply, 5, 3, 1),
                Instruction::new(Opcode::XorMultiply, 1, 0, 33),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 6, 19),
                Instruction::new(Opcode::RotateAdd, 6, 3, 61),
                Instruction::new(Opcode::AddMultiply, 2, 5, 1),
                Instruction::new(Opcode::RotateSub, 4, 7, 54),
                Instruction::new(Opcode::LogicalXor, 5, 2, 2),
                Instruction::new(Opcode::AddMultiply, 7, 4, 33),
                Instruction::new(Opcode::OrMultiply, 0, 1, 1),
                Instruction::new(Opcode::RotateAdd, 3, 0, 62),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 1, 50),
                Instruction::new(Opcode::RotateAdd, 6, 4, 35),
                Instruction::new(Opcode::OrMultiply, 5, 2, 1),
                Instruction::new(Opcode::XorMultiply, 1, 3, 1),
                Instruction::new(Opcode::ArithmeticAdd, 3, 6, 1),
                Instruction::new(Opcode::ArithmeticSub, 2, 5, 2),
                Instruction::new(Opcode::OrMultiply, 7, 0, 33),
                Instruction::new(Opcode::ArithmeticAdd, 0, 7, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 0, 31),
                Instruction::new(Opcode::XorMultiply, 6, 5, 33),
                Instruction::new(Opcode::ArithmeticSub, 1, 7, 1),
                Instruction::new(Opcode::OrMultiply, 0, 3, 9),
                Instruction::new(Opcode::RotateAdd, 2, 6, 36),
                Instruction::new(Opcode::RotateXor, 5, 4, 37),
                Instruction::new(Opcode::OrMultiply, 3, 1, 1),
                Instruction::new(Opcode::LogicalAdd, 7, 2, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 3, 58),
                Instruction::new(Opcode::LogicalSub, 6, 7, 3),
                Instruction::new(Opcode::XorMultiply, 7, 6, 33),
                Instruction::new(Opcode::LogicalAdd, 2, 5, 1),
                Instruction::new(Opcode::LogicalXor, 0, 4, 1),
                Instruction::new(Opcode::OrMultiply, 5, 0, 1),
                Instruction::new(Opcode::ArithmeticXor, 4, 2, 1),
                Instruction::new(Opcode::XorMultiply, 3, 1, 33),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 5, 34),
                Instruction::new(Opcode::ArithmeticSub, 6, 7, 2),
                Instruction::new(Opcode::RotateSub, 7, 4, 1),
                Instruction::new(Opcode::LogicalAdd, 4, 0, 2),
                Instruction::new(Opcode::XorMultiply, 0, 2, 9),
                Instruction::new(Opcode::LogicalXor, 5, 1, 2),
                Instruction::new(Opcode::OrMultiply, 2, 3, 33),
                Instruction::new(Opcode::XorMultiply, 3, 6, 33),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 2, 11),
                Instruction::new(Opcode::RotateXor, 7, 6, 55),
                Instruction::new(Opcode::ArithmeticXor, 3, 0, 3),
                Instruction::new(Opcode::LogicalXor, 0, 7, 1),
                Instruction::new(Opcode::AddMultiply, 2, 4, 1),
                Instruction::new(Opcode::ArithmeticXor, 4, 3, 3),
                Instruction::new(Opcode::XorMultiply, 5, 1, 9),
                Instruction::new(Opcode::XorMultiply, 1, 5, 33),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 0, 24),
                Instruction::new(Opcode::LogicalAdd, 4, 2, 1),
                Instruction::new(Opcode::AddMultiply, 3, 6, 1),
                Instruction::new(Opcode::OrMultiply, 7, 1, 1),
                Instruction::new(Opcode::LogicalAdd, 2, 4, 1),
                Instruction::new(Opcode::OrMultiply, 1, 3, 9),
                Instruction::new(Opcode::RotateXor, 6, 5, 42),
                Instruction::new(Opcode::ArithmeticXor, 0, 7, 3),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 1, 20),
                Instruction::new(Opcode::XorMultiply, 2, 7, 9),
                Instruction::new(Opcode::RotateSub, 6, 5, 44),
                Instruction::new(Opcode::ArithmeticXor, 5, 2, 3),
                Instruction::new(Opcode::ArithmeticXor, 1, 4, 2),
                Instruction::new(Opcode::ArithmeticSub, 3, 0, 3),
                Instruction::new(Opcode::OrMultiply, 0, 3, 33),
                Instruction::new(Opcode::AddMultiply, 4, 6, 9),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 7, 22),
                Instruction::new(Opcode::ArithmeticAdd, 0, 5, 2),
                Instruction::new(Opcode::LogicalAdd, 4, 6, 2),
                Instruction::new(Opcode::XorMultiply, 6, 2, 33),
                Instruction::new(Opcode::OrMultiply, 1, 0, 9),
                Instruction::new(Opcode::RotateXor, 7, 3, 26),
                Instruction::new(Opcode::RotateAdd, 3, 4, 1),
                Instruction::new(Opcode::XorMultiply, 5, 1, 33),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 7, 59),
                Instruction::new(Opcode::ArithmeticAdd, 1, 0, 2),
                Instruction::new(Opcode::LogicalSub, 2, 5, 1),
                Instruction::new(Opcode::XorMultiply, 0, 1, 1),
                Instruction::new(Opcode::OrMultiply, 3, 2, 9),
                Instruction::new(Opcode::AddMultiply, 5, 3, 33),
                Instruction::new(Opcode::ArithmeticXor, 7, 6, 2),
                Instruction::new(Opcode::LogicalAdd, 6, 4, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 7, 21),
                Instruction::new(Opcode::ArithmeticXor, 3, 6, 3),
                Instruction::new(Opcode::XorMultiply, 6, 1, 33),
                Instruction::new(Opcode::AddMultiply, 1, 5, 9),
                Instruction::new(Opcode::LogicalAdd, 7, 0, 3),
                Instruction::new(Opcode::RotateSub, 5, 4, 31),
                Instruction::new(Opcode::XorMultiply, 4, 2, 9),
                Instruction::new(Opcode::LogicalXor, 2, 3, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 4, 33),
                Instruction::new(Opcode::LogicalAdd, 7, 3, 2),
                Instruction::new(Opcode::LogicalAdd, 3, 0, 3),
                Instruction::new(Opcode::XorMultiply, 5, 1, 1),
                Instruction::new(Opcode::ArithmeticSub, 4, 6, 3),
                Instruction::new(Opcode::OrMultiply, 2, 7, 1),
                Instruction::new(Opcode::XorMultiply, 1, 2, 9),
                Instruction::new(Opcode::LogicalAdd, 6, 5, 2),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 3, 4),
                Instruction::new(Opcode::LogicalAdd, 5, 4, 2),
                Instruction::new(Opcode::LogicalAdd, 4, 5, 3),
                Instruction::new(Opcode::XorMultiply, 3, 7, 1),
                Instruction::new(Opcode::XorMultiply, 1, 2, 9),
                Instruction::new(Opcode::OrMultiply, 2, 1, 9),
                Instruction::new(Opcode::LogicalAdd, 6, 0, 2),
                Instruction::new(Opcode::RotateXor, 0, 6, 22),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 4, 3),
                Instruction::new(Opcode::LogicalSub, 1, 6, 1),
                Instruction::new(Opcode::OrMultiply, 4, 1, 1),
                Instruction::new(Opcode::OrMultiply, 5, 0, 33),
                Instruction::new(Opcode::RotateSub, 0, 3, 19),
                Instruction::new(Opcode::LogicalSub, 2, 7, 3),
                Instruction::new(Opcode::ArithmeticSub, 6, 5, 1),
                Instruction::new(Opcode::OrMultiply, 7, 2, 9),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 6, 3),
                Instruction::new(Opcode::OrMultiply, 3, 0, 9),
                Instruction::new(Opcode::AddMultiply, 7, 5, 33),
                Instruction::new(Opcode::OrMultiply, 2, 3, 9),
                Instruction::new(Opcode::LogicalSub, 0, 2, 3),
                Instruction::new(Opcode::LogicalAdd, 5, 1, 3),
                Instruction::new(Opcode::ArithmeticSub, 6, 4, 2),
                Instruction::new(Opcode::ArithmeticSub, 4, 7, 3),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 0, 29),
                Instruction::new(Opcode::OrMultiply, 0, 1, 9),
                Instruction::new(Opcode::XorMultiply, 5, 4, 9),
                Instruction::new(Opcode::RotateXor, 4, 5, 47),
                Instruction::new(Opcode::RotateAdd, 2, 3, 13),
                Instruction::new(Opcode::RotateSub, 3, 6, 51),
                Instruction::new(Opcode::XorMultiply, 7, 2, 33),
                Instruction::new(Opcode::RotateXor, 1, 7, 31),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 6, 52),
                Instruction::new(Opcode::RotateXor, 7, 3, 39),
                Instruction::new(Opcode::RotateSub, 3, 4, 56),
                Instruction::new(Opcode::ArithmeticSub, 4, 5, 3),
                Instruction::new(Opcode::OrMultiply, 5, 2, 1),
                Instruction::new(Opcode::LogicalSub, 6, 0, 3),
                Instruction::new(Opcode::AddMultiply, 2, 1, 33),
                Instruction::new(Opcode::OrMultiply, 1, 7, 1),
            ],
            unconditional: false,
        },
        Program {
            instructions: [
                Instruction::new(Opcode::MultiplyRotate, 8, 7, 5),
                Instruction::new(Opcode::LogicalXor, 1, 6, 2),
                Instruction::new(Opcode::OrMultiply, 5, 4, 9),
                Instruction::new(Opcode::OrMultiply, 0, 2, 1),
                Instruction::new(Opcode::ArithmeticXor, 2, 5, 3),
                Instruction::new(Opcode::RotateXor, 4, 0, 62),
                Instruction::new(Opcode::AddMultiply, 6, 1, 1),
                Instruction::new(Opcode::ArithmeticSub, 7, 3, 2),
            ],
            unconditional: true,
        },
    ]
};

const K0: u64 = 0x1716_1514_1312_1110;
const K1: u64 = 0x1f1e_1d1c_1b1a_1918;
const EXPECTED: u64 = 0xfd5d_450b_7670_35c5;

#[expect(
    clippy::used_underscore_binding,
    reason = "dynasm 5.1 references its own underscore-prefixed temporary bindings"
)]
fn emit_sip(ops: &mut dynasmrt::x64::Assembler, a: Rq, b: Rq, c: Rq, d: Rq) {
    dynasm!(ops;
     add Rq(a),Rq(b); add Rq(c),Rq(d); rol Rq(b),13; rol Rq(d),16; xor Rq(b),Rq(a); xor Rq(d),Rq(c); rol Rq(a),32;
     add Rq(c),Rq(b); add Rq(a),Rq(d); rol Rq(b),17; rol Rq(d),21; xor Rq(b),Rq(c); xor Rq(d),Rq(a); rol Rq(c),32
    );
}
struct Jit {
    buf: dynasmrt::ExecutableBuffer,
    entry: unsafe extern "sysv64" fn(u64) -> u64,
}
impl Jit {
    fn compile() -> Result<Self, String> {
        let mut ops = dynasmrt::x64::Assembler::new().map_err(|err| err.to_string())?;
        dynasm!(ops ; .arch x64 ; push rbx ; push r12 ; push r13 ; push r14 ; push r15
  ; mov r8,QWORD i64::from_ne_bytes((0x736f_6d65_7073_6575_u64^K0).to_ne_bytes())
  ; mov r9,QWORD i64::from_ne_bytes((0x646f_7261_6e64_6f6d_u64^K1).to_ne_bytes())
  ; mov r10,QWORD i64::from_ne_bytes((0x6c79_6765_6e65_7261_u64^K0).to_ne_bytes())
  ; mov r11,QWORD i64::from_ne_bytes((0x7465_6462_7974_6573_u64^K1).to_ne_bytes())
  ; xor r11,rdi);
        emit_sip(&mut ops, Rq::R8, Rq::R9, Rq::R10, Rq::R11);
        dynasm!(ops; xor r8,rdi; xor r10,0xbb);
        for _ in 0..3 {
            emit_sip(&mut ops, Rq::R8, Rq::R9, Rq::R10, Rq::R11);
        }
        dynasm!(ops; mov r12,r11; mov r13,r10; mov r14,r9; mov r15,r8
  ; mov rax,QWORD i64::from_ne_bytes(K0.to_ne_bytes()); xor r8,rax; xor r10,rax
  ; mov rax,QWORD i64::from_ne_bytes(K1.to_ne_bytes()); xor r9,rax; xor r11,rax);
        for _ in 0..4 {
            emit_sip(&mut ops, Rq::R8, Rq::R9, Rq::R10, Rq::R11);
        }
        dynasm!(ops; mov rax,r11; mov rcx,r10; mov rdx,r9; mov rsi,r8
  ; mov r8,r12; mov r9,r13; mov r10,r14; mov r11,r15
  ; mov r12,rax; mov r13,rcx; mov r14,rdx; mov r15,rsi
  ; mov rbx,r8; xor rbx,r12; and rbx,-8; or rbx,3; push rbx);
        emit_phase(&mut ops, false);
        emit_phase(&mut ops, true);
        dynasm!(ops; add rsp,16392);
        emit_sip(&mut ops, Rq::R8, Rq::R9, Rq::R10, Rq::R11);
        emit_sip(&mut ops, Rq::R12, Rq::R13, Rq::R14, Rq::R15);
        dynasm!(ops; mov rax,r11; xor rax,r15; xor rax,rbx; pop r15; pop r14; pop r13; pop r12; pop rbx; ret);
        let buf = ops
            .finalize()
            .map_err(|_| "could not finalize native emitter")?;
        // SAFETY: the emitter above creates one x86-64 System V function at
        // offset zero, taking and returning one u64. The owning executable
        // buffer remains alive for every invocation.
        let entry = unsafe {
            std::mem::transmute::<*const u8, unsafe extern "sysv64" fn(u64) -> u64>(
                buf.ptr(dynasmrt::AssemblyOffset(0)),
            )
        };
        Ok(Self { buf, entry })
    }

    #[inline]
    fn hash(&self, salt: u64) -> u64 {
        // SAFETY: entry is emitted and finalized by compile(), and its owner
        // remains live. The fixed program uses registers 0..8 and bounded
        // shift counts; the final unconditional program consumes the remaining
        // branch budget, so the first phase writes exactly 256 stack snapshots.
        // The second phase reads from that populated 16 KiB scratch and the
        // adjacent eight-byte word.
        // The prologue/epilogue preserve callee-saved registers and restore RSP.
        unsafe { (self.entry)(salt) }
    }
}

// The fixed instruction stream uses eight working registers and r8's
// dedicated multiplier. Conversion happens at compile time in the fixture.
const fn register(index: u8) -> Rq {
    const REGISTERS: [Rq; 9] = [
        Rq::R8,
        Rq::R9,
        Rq::R10,
        Rq::R11,
        Rq::R12,
        Rq::R13,
        Rq::R14,
        Rq::R15,
        Rq::RBX,
    ];
    REGISTERS[index as usize]
}

#[expect(
    clippy::used_underscore_binding,
    reason = "dynasm 5.1 references its own underscore-prefixed temporary bindings"
)]
fn source(ops: &mut dynasmrt::x64::Assembler, src: Rq) {
    dynasm!(ops ; mov eax,Rd(src) ; and eax,16383);
}
enum Combine {
    Multiply,
    Xor,
    Add,
    Sub,
}

#[expect(
    clippy::used_underscore_binding,
    reason = "dynasm 5.1 references its own underscore-prefixed temporary bindings"
)]
fn emit_op(ops: &mut dynasmrt::x64::Assembler, ins: Instruction, memory: bool) {
    let dst = ins.dst;
    let src = ins.src;
    let imm = ins.imm;
    let combine = match ins.op {
        Opcode::OrMultiply => {
            dynasm!(ops; or Rq(dst), i32::from(imm));
            Combine::Multiply
        }
        Opcode::XorMultiply => {
            dynasm!(ops; xor Rq(dst), i32::from(imm));
            Combine::Multiply
        }
        Opcode::AddMultiply => {
            dynasm!(ops; add Rq(dst), i32::from(imm));
            Combine::Multiply
        }
        Opcode::MultiplyRotate => {
            dynasm!(ops; imul Rq(dst),rbx; ror Rq(dst),imm);
            return;
        }
        Opcode::RotateXor => {
            dynasm!(ops; ror Rq(dst),imm);
            Combine::Xor
        }
        Opcode::RotateAdd => {
            dynasm!(ops; ror Rq(dst),imm);
            Combine::Add
        }
        Opcode::RotateSub => {
            dynasm!(ops; ror Rq(dst),imm);
            Combine::Sub
        }
        Opcode::ArithmeticXor => {
            dynasm!(ops; sar Rq(dst),imm);
            Combine::Xor
        }
        Opcode::ArithmeticAdd => {
            dynasm!(ops; sar Rq(dst),imm);
            Combine::Add
        }
        Opcode::ArithmeticSub => {
            dynasm!(ops; sar Rq(dst),imm);
            Combine::Sub
        }
        Opcode::LogicalXor => {
            dynasm!(ops; shr Rq(dst),imm);
            Combine::Xor
        }
        Opcode::LogicalAdd => {
            dynasm!(ops; shr Rq(dst),imm);
            Combine::Add
        }
        Opcode::LogicalSub => {
            dynasm!(ops; shr Rq(dst),imm);
            Combine::Sub
        }
    };
    if memory {
        source(ops, ins.src);
        match combine {
            Combine::Multiply => dynasm!(ops; imul Rq(dst),QWORD [rsp+rax]),
            Combine::Xor => dynasm!(ops; xor Rq(dst),QWORD [rsp+rax]),
            Combine::Add => dynasm!(ops; add Rq(dst),QWORD [rsp+rax]),
            Combine::Sub => dynasm!(ops; sub Rq(dst),QWORD [rsp+rax]),
        }
    } else {
        match combine {
            Combine::Multiply => dynasm!(ops; imul Rq(dst),Rq(src)),
            Combine::Xor => dynasm!(ops; xor Rq(dst),Rq(src)),
            Combine::Add => dynasm!(ops; add Rq(dst),Rq(src)),
            Combine::Sub => dynasm!(ops; sub Rq(dst),Rq(src)),
        }
    }
}
#[expect(
    clippy::used_underscore_binding,
    reason = "dynasm 5.1 references its own underscore-prefixed temporary bindings"
)]
fn emit_phase(ops: &mut dynasmrt::x64::Assembler, mem: bool) {
    let rep = ops.new_dynamic_label();
    dynasm!(ops; mov esi,4 ; =>rep ; mov edx,32);
    for p in &PROGRAMS {
        let start = ops.new_dynamic_label();
        let done = ops.new_dynamic_label();
        dynasm!(ops; =>start);
        if !mem {
            dynasm!(ops; push r8 ; push r9 ; push r10 ; push r11 ; push r12 ; push r13 ; push r14 ; push r15);
        }
        for x in p.instructions {
            emit_op(ops, x, mem);
        }
        let bf = p.instructions[0].dst;
        dynasm!(ops; test edx,edx ; jz =>done);
        if !p.unconditional {
            dynasm!(ops; test Rd(bf),32 ; jnz =>done);
        }
        dynasm!(ops; dec edx ; jmp =>start ; =>done);
    }
    dynasm!(ops; dec esi ; jnz =>rep);
}
fn bench_count(raw: &str, minimum: u32) -> Result<u32, String> {
    let value = raw
        .parse::<u32>()
        .map_err(|_| format!("invalid decimal count: {raw}"))?;
    if !raw.bytes().all(|b| b.is_ascii_digit()) || !(minimum..=2_147_483_647).contains(&value) {
        return Err(format!("benchmark count out of range: {raw}"));
    }
    Ok(value)
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [warmup, iterations, samples] = args.as_slice() else {
        return Err("usage: museum-hashwx-dynasm WARMUP ITERATIONS SAMPLES".into());
    };
    let warmup = bench_count(warmup, 0)?;
    let iterations = bench_count(iterations, 1)?;
    let samples = bench_count(samples, 1)?;
    println!("BENCHMARK_DIMENSIONS v1 {warmup} {iterations} {samples}");
    let t = Instant::now();
    let j = Jit::compile()?;
    let build = t.elapsed();
    let check = j.hash(123_456);
    if check != EXPECTED {
        return Err(format!("incorrect hash: {check:016x} != {EXPECTED:016x}"));
    }
    println!(
        "BUILD_US {:.3} CODE {} CHECK {check:016x}",
        build.as_secs_f64() * 1e6,
        j.buf.len()
    );
    for i in 0..warmup {
        std::hint::black_box(j.hash(u64::from(i)));
    }
    for _ in 0..samples {
        let mut sink = 0u64;
        let t = Instant::now();
        for i in 0..iterations {
            sink ^= j.hash(u64::from(i));
        }
        let sec = t.elapsed().as_secs_f64();
        println!("RATE {:.3} sink={sink:016x}", f64::from(iterations) / sec);
    }
    Ok(())
}
