use std::fmt::Write as _;

use hashwx::{HashXInstruction, HashXProgram, Op, ProgramSet};

fn word(value: u64) -> String {
    let [lo0, lo1, lo2, lo3, hi0, hi1, hi2, hi3] = value.to_le_bytes();
    format!(
        "Word({}u, {}u)",
        u32::from_le_bytes([lo0, lo1, lo2, lo3]),
        u32::from_le_bytes([hi0, hi1, hi2, hi3])
    )
}

fn header() -> String {
    let mut out = include_str!("words.wgsl").to_owned();
    out.push_str(
        "@group(0) @binding(0) var<storage, read> inputs: array<Word>;\n\
         @group(0) @binding(1) var<storage, read_write> outputs: array<Word>;\n",
    );
    out
}

fn entry(out: &mut String, scratch: bool) {
    let argument = if scratch { ", gid.x" } else { "" };
    writeln!(
        out,
        "@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
         if (gid.x >= arrayLength(&inputs)) {{ return; }}\n\
         outputs[gid.x] = hash(inputs[gid.x]{argument});\n}}"
    )
    .unwrap();
}

pub fn hashx(program: &HashXProgram) -> String {
    let mut out = header();
    let keys = program.register_key().map(word);
    writeln!(
        out,
        "fn hash(nonce: Word) -> Word {{\n\
         var key = State({}, {}, {}, {});\n\
         var v = key;\n\
         v[1] ^= Word(238u, 0u); v[3] ^= nonce; v = sip(sip(v));\n\
         v[0] ^= nonce; v[2] ^= Word(238u, 0u); v = sip(sip(sip(sip(v))));\n\
         var t = v; t[1] ^= Word(221u, 0u); t = sip(sip(sip(sip(t))));\n\
         var r = array<Word, 8>(v[0],v[1],v[2],v[3],t[0],t[1],t[2],t[3]);\n\
         var allowed = true; var mulh_result = 0u;",
        keys[0], keys[1], keys[2], keys[3]
    )
    .unwrap();
    let mut target = false;
    for instruction in program.instructions() {
        use HashXInstruction as I;
        let line = match *instruction {
            I::Mul { dst, src } => format!(
                "r[{}] = mul64(r[{}], r[{}]);",
                dst.index(),
                dst.index(),
                src.index()
            ),
            I::UMulH { dst, src } | I::SMulH { dst, src } => {
                let d = dst.index();
                let op = if matches!(instruction, I::UMulH { .. }) {
                    "umulh"
                } else {
                    "smulh"
                };
                format!(
                    "r[{d}] = {op}(r[{d}], r[{}]); mulh_result = r[{d}].x;",
                    src.index()
                )
            }
            I::AddShift {
                dst,
                src,
                left_shift,
            } => format!(
                "r[{}] = add64(r[{}], shl64(r[{}], {left_shift}u));",
                dst.index(),
                dst.index(),
                src.index()
            ),
            I::AddConst { dst, src } => format!(
                "r[{}] = add64(r[{}], {});",
                dst.index(),
                dst.index(),
                word(src as i64 as u64)
            ),
            I::Sub { dst, src } => format!(
                "r[{}] = sub64(r[{}], r[{}]);",
                dst.index(),
                dst.index(),
                src.index()
            ),
            I::Xor { dst, src } => format!("r[{}] ^= r[{}];", dst.index(), src.index()),
            I::XorConst { dst, src } => {
                format!("r[{}] ^= {};", dst.index(), word(src as i64 as u64))
            }
            I::Rotate { dst, right_rotate } => format!(
                "r[{}] = ror64(r[{}], {right_rotate}u);",
                dst.index(),
                dst.index()
            ),
            I::Target => {
                assert!(!target);
                target = true;
                "loop {".to_owned()
            }
            I::Branch { mask } => {
                assert!(target);
                target = false;
                format!(
                    "if (!allowed || (mulh_result & {mask}u) != 0u) {{ break; }} allowed = false; }}"
                )
            }
        };
        writeln!(out, "{line}").unwrap();
    }
    assert!(!target);
    out.push_str(
        "let x = sip(State(add64(r[0],key[0]),add64(r[1],key[1]),r[2],r[3]));\n\
         let y = sip(State(r[4],r[5],add64(r[6],key[2]),add64(r[7],key[3])));\n\
         return x[0] ^ y[0];\n}\n",
    );
    entry(&mut out, false);
    out
}

pub fn hashwx(program: &ProgramSet) -> String {
    let mut out = header();
    out.push_str(
        "@group(0) @binding(2) var<storage, read_write> scratch: array<u32>;\n\
         fn load_word(base: u32, address: u32) -> Word {\n\
           let pos = base + address / 4u; let shift = (address & 3u) * 8u;\n\
           let a = scratch[pos]; let b = scratch[pos + 1u];\n\
           if (shift == 0u) { return Word(a,b); }\n\
           return Word((a >> shift) | (b << (32u-shift)), (b >> shift) | (scratch[pos+2u] << (32u-shift)));\n\
         }\n\
         fn store_word(base: u32, address: u32, value: Word) {\n\
           let pos = base + address / 4u; scratch[pos] = value.x; scratch[pos+1u] = value.y;\n\
         }\n",
    );
    let [k0, k1] = program.hash_key();
    writeln!(
        out,
        "fn hash(nonce: Word, lane: u32) -> Word {{\n\
         let base = lane * 4098u;\n\
         let k0 = {}; let k1 = {};\n\
         var v = State({}, {}, {}, {});\n\
         v[3] ^= nonce; v = sip(v); v[0] ^= nonce; v[2] ^= Word(187u,0u);\n\
         v = sip(sip(sip(v)));\n\
         var r = array<Word,9>(v[3],v[2],v[1],v[0],Word(0u),Word(0u),Word(0u),Word(0u),Word(0u));\n\
         v[0] ^= k0; v[1] ^= k1; v[2] ^= k0; v[3] ^= k1; v = sip(sip(sip(sip(v))));\n\
         r[4] = v[3]; r[5] = v[2]; r[6] = v[1]; r[7] = v[0];\n\
         r[8] = ((r[0] ^ r[4]) & Word(0xfffffff8u,0xffffffffu)) | Word(3u,0u);\n\
         store_word(base,16384u,r[8]); var cursor = 16384u;",
        word(k0),
        word(k1),
        word(0x736f_6d65_7073_6575 ^ k0),
        word(0x646f_7261_6e64_6f6d ^ k1),
        word(0x6c79_6765_6e65_7261 ^ k0),
        word(0x7465_6462_7974_6573 ^ k1)
    )
    .unwrap();
    for memory in [false, true] {
        out.push_str("for (var rep=0u; rep<4u; rep++) { var budget=32u;\n");
        for p in program.programs() {
            out.push_str("loop {\n");
            if !memory {
                out.push_str("cursor -= 64u;\n");
                for reg in 0..8 {
                    writeln!(out, "store_word(base,cursor+{}u,r[{reg}]);", 56 - reg * 8).unwrap();
                }
            }
            for ins in p.body() {
                let d = format!("r[{}]", ins.dst());
                let s = if memory {
                    format!("load_word(base, r[{}].x & 16383u)", ins.src())
                } else {
                    format!("r[{}]", ins.src())
                };
                let imm = ins.imm();
                let expr = match ins.op() {
                    Op::MulOr => format!("mul64({d} | Word({imm}u,0u), {s})"),
                    Op::MulXor => format!("mul64({d} ^ Word({imm}u,0u), {s})"),
                    Op::MulAdd => format!("mul64(add64({d},Word({imm}u,0u)), {s})"),
                    Op::Rmcg => format!("ror64(mul64({d},r[8]),{imm}u)"),
                    op => {
                        use Op::*;
                        let shift = match op {
                            XorRor | AddRor | SubRor => "ror64",
                            XorAsr | AddAsr | SubAsr => "asr64",
                            XorLsr | AddLsr | SubLsr => "shr64",
                            _ => unreachable!(),
                        };
                        let left = format!("{shift}({d},{imm}u)");
                        match op {
                            XorRor | XorAsr | XorLsr => format!("{left} ^ {s}"),
                            AddRor | AddAsr | AddLsr => format!("add64({left},{s})"),
                            SubRor | SubAsr | SubLsr => format!("sub64({left},{s})"),
                            _ => unreachable!(),
                        }
                    }
                };
                writeln!(out, "{d} = {expr};").unwrap();
            }
            if !p.unconditional() {
                writeln!(
                    out,
                    "if ((r[{}].x & 32u) != 0u) {{ break; }}",
                    p.body()[0].dst()
                )
                .unwrap();
            }
            out.push_str("if (budget == 0u) { break; } budget--; }\n");
        }
        out.push_str("}\n");
    }
    out.push_str(
        "let lo = sip(State(r[0],r[1],r[2],r[3]));\n\
         let hi = sip(State(r[4],r[5],r[6],r[7]));\n\
         return lo[3] ^ hi[3] ^ r[8];\n}\n",
    );
    entry(&mut out, true);
    out
}

#[cfg(test)]
mod tests {
    use super::{hashwx, hashx};
    use hashwx::{HashXProgram, generate};

    #[test]
    fn generated_shaders_parse_and_validate() {
        for seed in [[0; 32], [1; 32], [255; 32], super::super::SEED] {
            let mut shaders = vec![hashwx(&generate(&seed))];
            if let Ok(program) = HashXProgram::generate(&seed) {
                shaders.push(hashx(&program));
            }
            for source in shaders {
                let module = naga::front::wgsl::parse_str(&source)
                    .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
                naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::empty(),
                )
                .validate(&module)
                .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
            }
        }
    }
}
