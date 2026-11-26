//! Lower a complete `HashWX` `ProgramSet` to native-targetable CUDA PTX without NVRTC.
//! Native code is produced by one ptxas invocation per seed, allowing us to measure
//! the backend compiler cost separately from the expensive CUDA C++ front end.

use hashwx::{Instruction, Op, ProgramSet};
use std::fmt::{self, Write};

struct Emitter {
    text: String,
    label: u32,
}

impl Emitter {
    const fn new() -> Self {
        Self {
            text: String::new(),
            label: 0,
        }
    }

    fn ror(&mut self, register: &str, amount: u8) -> fmt::Result {
        let complement = 64_u8 - amount;
        writeln!(self.text, "shr.u64 %z0,{register},{amount};")?;
        writeln!(self.text, "shl.b64 %z1,{register},{complement};")?;
        writeln!(self.text, "or.b64 {register},%z0,%z1;")
    }

    fn sip_round(&mut self) -> fmt::Result {
        self.text
            .push_str("add.u64 %v0,%v0,%v1;\nadd.u64 %v2,%v2,%v3;\n");
        self.ror("%v1", 51)?;
        self.ror("%v3", 48)?;
        self.text
            .push_str("xor.b64 %v1,%v1,%v0;\nxor.b64 %v3,%v3,%v2;\n");
        self.ror("%v0", 32)?;
        self.text
            .push_str("add.u64 %v2,%v2,%v1;\nadd.u64 %v0,%v0,%v3;\n");
        self.ror("%v1", 47)?;
        self.ror("%v3", 43)?;
        self.text
            .push_str("xor.b64 %v1,%v1,%v2;\nxor.b64 %v3,%v3,%v0;\n");
        self.ror("%v2", 32)
    }

    fn snapshot(&mut self) {
        self.text.push_str(
            "sub.u32 %u6,%u6,8;\n\
             mul.wide.u32 %a1,%u6,8;\n\
             add.u64 %a2,%a0,%a1;\n\
             st.shared.u64 [%a2+56],%r0;\n\
             st.shared.u64 [%a2+48],%r1;\n\
             st.shared.u64 [%a2+40],%r2;\n\
             st.shared.u64 [%a2+32],%r3;\n\
             st.shared.u64 [%a2+24],%r4;\n\
             st.shared.u64 [%a2+16],%r5;\n\
             st.shared.u64 [%a2+8],%r6;\n\
             st.shared.u64 [%a2],%r7;\n",
        );
    }

    fn read_scratch(&mut self, register: u8) -> fmt::Result {
        writeln!(
            self.text,
            "cvt.u32.u64 %u7,%r{register};\n\
             and.b32 %u8,%u7,16383;\n\
             shr.u32 %u9,%u8,3;\n\
             mul.wide.u32 %a1,%u9,8;\n\
             add.u64 %a2,%a0,%a1;\n\
             ld.shared.u64 %a3,[%a2];\n\
             ld.shared.u64 %a4,[%a2+8];\n\
             and.b32 %u7,%u8,7;\n\
             mul.lo.u32 %u7,%u7,8;\n\
             shr.u64 %a3,%a3,%u7;\n\
             sub.u32 %u8,64,%u7;\n\
             and.b32 %u8,%u8,63;\n\
             shl.b64 %a4,%a4,%u8;\n\
             setp.ne.u32 %p2,%u7,0;\n\
             selp.b64 %a4,%a4,0,%p2;\n\
             or.b64 %z2,%a3,%a4;"
        )
    }

    fn instruction(&mut self, instruction: Instruction, memory_phase: bool) -> fmt::Result {
        let d = instruction.dst();
        let s = instruction.src();
        let imm = instruction.imm();
        let operation = instruction.op();

        if memory_phase && operation != Op::Rmcg {
            self.read_scratch(s)?;
        }
        let operand = if memory_phase && operation != Op::Rmcg {
            "%z2".to_owned()
        } else {
            format!("%r{s}")
        };
        match operation {
            Op::MulOr => writeln!(
                self.text,
                "or.b64 %z0,%r{d},{imm};\nmul.lo.u64 %r{d},%z0,{operand};"
            ),
            Op::MulXor => writeln!(
                self.text,
                "xor.b64 %z0,%r{d},{imm};\nmul.lo.u64 %r{d},%z0,{operand};"
            ),
            Op::MulAdd => writeln!(
                self.text,
                "add.u64 %z0,%r{d},{imm};\nmul.lo.u64 %r{d},%z0,{operand};"
            ),
            Op::Rmcg => {
                writeln!(self.text, "mul.lo.u64 %r{d},%r{d},%r8;")?;
                self.ror(&format!("%r{d}"), imm)
            }
            Op::XorRor
            | Op::AddRor
            | Op::SubRor
            | Op::XorAsr
            | Op::AddAsr
            | Op::SubAsr
            | Op::XorLsr
            | Op::AddLsr
            | Op::SubLsr => {
                match operation {
                    Op::XorRor | Op::AddRor | Op::SubRor => {
                        writeln!(self.text, "mov.b64 %z3,%r{d};")?;
                        self.ror("%z3", imm)?;
                    }
                    Op::XorAsr | Op::AddAsr | Op::SubAsr => {
                        writeln!(self.text, "shr.s64 %z3,%r{d},{imm};")?;
                    }
                    _ => {
                        writeln!(self.text, "shr.u64 %z3,%r{d},{imm};")?;
                    }
                }
                let op = match operation {
                    Op::XorRor | Op::XorAsr | Op::XorLsr => "xor.b64",
                    Op::AddRor | Op::AddAsr | Op::AddLsr => "add.u64",
                    _ => "sub.u64",
                };
                writeln!(self.text, "{op} %r{d},%z3,{operand};")
            }
        }
    }

    fn phase(&mut self, program: &ProgramSet, memory_phase: bool) -> fmt::Result {
        let rep_start = self.label;
        self.label += 1;
        writeln!(self.text, "mov.u32 %u4,0;\nL{rep_start}:")?;
        self.text.push_str("mov.u32 %u5,32;\n");
        for subprogram in program.programs() {
            let begin = self.label;
            let end = self.label + 1;
            self.label += 2;
            writeln!(self.text, "L{begin}:")?;
            if !memory_phase {
                self.snapshot();
            }
            for &instruction in subprogram.body() {
                self.instruction(instruction, memory_phase)?;
            }
            if !subprogram.unconditional() {
                let dst = subprogram.body()[0].dst();
                writeln!(
                    self.text,
                    "and.b64 %z0,%r{dst},32;\n\
                     setp.ne.u64 %p1,%z0,0;\n\
                     @%p1 bra L{end};"
                )?;
            }
            writeln!(
                self.text,
                "setp.eq.u32 %p1,%u5,0;\n\
                 @%p1 bra L{end};\n\
                 sub.u32 %u5,%u5,1;\n\
                 bra L{begin};\n\
                 L{end}:"
            )?;
        }
        writeln!(
            self.text,
            "add.u32 %u4,%u4,1;\n\
             setp.lt.u32 %p1,%u4,4;\n\
             @%p1 bra L{rep_start};"
        )
    }
}

pub fn emit(program: &ProgramSet) -> Result<String, fmt::Error> {
    let mut e = Emitter::new();
    e.text.push_str(
        ".version 8.8\n.target sm_86\n.address_size 64\n\
         .extern .shared .align 8 .b8 scratch[];\n\
         .visible .entry hash_batch(\n\
           .param .u64 input, .param .u64 output, .param .u32 count)\n\
         .maxntid 2,1,1\n\
         {\n\
         .reg .pred %p<4>;\n\
         .reg .b32 %u<12>;\n\
         .reg .b64 %r<9>;\n\
         .reg .b64 %v<4>;\n\
         .reg .b64 %a<5>;\n\
         .reg .b64 %g<2>;\n\
         .reg .b64 %z<4>;\n\
         ld.param.u64 %g0,[input];\n\
         ld.param.u64 %g1,[output];\n\
         ld.param.u32 %u3,[count];\n\
         mov.u32 %u0,%tid.x;\n\
         mov.u32 %u1,%ctaid.x;\n\
         mad.lo.u32 %u2,%u1,2,%u0;\n\
         setp.ge.u32 %p0,%u2,%u3;\n\
         @%p0 bra END;\n\
         mul.wide.u32 %a1,%u2,8;\n\
         add.u64 %a2,%g0,%a1;\n\
         ld.global.u64 %z2,[%a2];\n\
         mov.u64 %a0,scratch;\n\
         mul.wide.u32 %a1,%u0,16392;\n\
         add.u64 %a0,%a0,%a1;\n",
    );
    let [k0, k1] = program.hash_key();
    let v = [
        0x736f_6d65_7073_6575_u64 ^ k0,
        0x646f_7261_6e64_6f6d_u64 ^ k1,
        0x6c79_6765_6e65_7261_u64 ^ k0,
        0x7465_6462_7974_6573_u64 ^ k1,
    ];
    for (index, value) in v.iter().enumerate() {
        writeln!(e.text, "mov.u64 %v{index},{value};")?;
    }
    e.text.push_str("xor.b64 %v3,%v3,%z2;\n");
    e.sip_round()?;
    e.text
        .push_str("xor.b64 %v0,%v0,%z2;\nxor.b64 %v2,%v2,187;\n");
    for _ in 0..3 {
        e.sip_round()?;
    }
    for (index, src) in [3, 2, 1, 0].into_iter().enumerate() {
        writeln!(e.text, "mov.b64 %r{index},%v{src};")?;
    }
    for (index, key) in [k0, k1, k0, k1].into_iter().enumerate() {
        writeln!(e.text, "xor.b64 %v{index},%v{index},{key};")?;
    }
    for _ in 0..4 {
        e.sip_round()?;
    }
    for (index, src) in [3, 2, 1, 0].into_iter().enumerate() {
        writeln!(e.text, "mov.b64 %r{},%v{src};", index + 4)?;
    }
    e.text.push_str(
        "xor.b64 %r8,%r0,%r4;\n\
         and.b64 %r8,%r8,0xfffffffffffffff8;\n\
         or.b64 %r8,%r8,3;\n\
         st.shared.u64 [%a0+16384],%r8;\n\
         mov.u32 %u6,2048;\n",
    );
    e.phase(program, false)?;
    e.phase(program, true)?;
    e.text.push_str(
        "mov.b64 %v0,%r0;\nmov.b64 %v1,%r1;\n\
         mov.b64 %v2,%r2;\nmov.b64 %v3,%r3;\n",
    );
    e.sip_round()?;
    e.text.push_str("mov.b64 %z3,%v3;\n");
    for (index, src) in [4, 5, 6, 7].into_iter().enumerate() {
        writeln!(e.text, "mov.b64 %v{index},%r{src};")?;
    }
    e.sip_round()?;
    e.text.push_str(
        "xor.b64 %z3,%z3,%v3;\n\
         xor.b64 %z3,%z3,%r8;\n\
         mul.wide.u32 %a1,%u2,8;\n\
         add.u64 %a2,%g1,%a1;\n\
         st.global.u64 [%a2],%z3;\n\
         END:\nret;\n}\n",
    );
    Ok(e.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ptx_has_all_instruction_variants() -> fmt::Result {
        let mut observed_ops = [false; 13];
        for seed_index in 0..64_u8 {
            let seed_bytes = std::array::from_fn(|offset| {
                u8::try_from(offset)
                    .unwrap_or_default()
                    .wrapping_add(seed_index)
            });
            let program = hashwx::generate(&seed_bytes);
            let ptx = emit(&program)?;
            assert!(ptx.starts_with(".version"));
            assert_eq!(ptx.matches("mov.u32 %u5,32").count(), 2);
            for subprogram in program.programs() {
                for &ins in subprogram.body() {
                    observed_ops[ins.op() as usize] = true;
                }
            }
        }
        assert!(observed_ops.into_iter().all(|s| s));
        Ok(())
    }
}
