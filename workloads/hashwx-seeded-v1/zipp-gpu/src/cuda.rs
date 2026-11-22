//! Fixed-program CUDA C++ reference, compiled offline or through NVRTC.
//! The input/output ABI is the same u64 corpus as the WGSL implementations.
use std::{
    error::Error,
    fmt::Write as _,
    fs,
    io::{BufWriter, Write as _},
    path::Path,
};

use hashwx::{HashXInstruction as I, HashXProgram, Op, ProgramSet, generate};

use crate::{Algorithm, Case};

const COMMON: &str = include_str!("native.cuh");

fn hashx(program: &HashXProgram) -> Result<String, Box<dyn Error>> {
    let mut out = COMMON.to_owned();
    let [k0, k1, k2, k3] = program.register_key();
    writeln!(
        out,
        "
__device__ __forceinline__ U64 hash(U64 nonce, U64*) {{
    constexpr State key{{{k0}ULL,{k1}ULL,{k2}ULL,{k3}ULL}};
    State v=key;
    v.b ^= 238; v.d ^= nonce; v=sip(sip(v));
    v.a ^= nonce; v.c ^= 238; v=sip(sip(sip(sip(v))));
    State t=v; t.b ^= 221; t=sip(sip(sip(sip(t))));
    U64 r[8]={{v.a,v.b,v.c,v.d,t.a,t.b,t.c,t.d}};
    bool allowed=true; unsigned mh=0;
"
    )?;
    let mut target = false;
    for ins in program.instructions() {
        let line = match *ins {
            I::Mul { dst, src } => format!("r[{}] *= r[{}];", dst.index(), src.index()),
            I::UMulH { dst, src } | I::SMulH { dst, src } => {
                let d = dst.index();
                let s = src.index();
                let mul = if matches!(ins, I::UMulH { .. }) {
                    "umulhi"
                } else {
                    "smulhi"
                };
                format!("r[{d}]={mul}(r[{d}],r[{s}]); mh=unsigned(r[{d}]);")
            }
            I::AddShift {
                dst,
                src,
                left_shift,
            } => format!("r[{}] += r[{}] << {left_shift};", dst.index(), src.index()),
            I::AddConst { dst, src } => format!(
                "r[{}] += {}ULL;",
                dst.index(),
                i64::from(src).cast_unsigned()
            ),
            I::Sub { dst, src } => format!("r[{}] -= r[{}];", dst.index(), src.index()),
            I::Xor { dst, src } => format!("r[{}] ^= r[{}];", dst.index(), src.index()),
            I::XorConst { dst, src } => format!(
                "r[{}] ^= {}ULL;",
                dst.index(),
                i64::from(src).cast_unsigned()
            ),
            I::Rotate { dst, right_rotate } => {
                format!("r[{0}]=ror(r[{0}],{right_rotate});", dst.index())
            }
            I::Target => {
                if target {
                    return Err("nested HashX target".into());
                }
                target = true;
                "for (;;) {".to_owned()
            }
            I::Branch { mask } => {
                if !target {
                    return Err("HashX branch without target".into());
                }
                target = false;
                format!("if (!allowed || (mh & {mask}U)) break; allowed=false; }}")
            }
        };
        writeln!(out, "{line}")?;
    }
    if target {
        return Err("unclosed HashX target".into());
    }
    out.push_str(
        "
    State x=sip({r[0]+key.a,r[1]+key.b,r[2],r[3]});
    State y=sip({r[4],r[5],r[6]+key.c,r[7]+key.d});
    return x.a ^ y.a;
}
",
    );
    out.push_str(include_str!("native_entry.cuh"));
    Ok(out)
}

fn hashwx(program: &ProgramSet) -> Result<String, Box<dyn Error>> {
    let mut out = COMMON.to_owned();
    let [k0, k1] = program.hash_key();
    writeln!(out,"
#define HASHWX_SCRATCH 2049
__device__ __forceinline__ U64 hash(U64 nonce, U64* scratch) {{
    constexpr U64 k0={k0}ULL, k1={k1}ULL;
    State v{{0x736f6d6570736575ULL^k0,0x646f72616e646f6dULL^k1,0x6c7967656e657261ULL^k0,0x7465646279746573ULL^k1}};
    v.d ^= nonce; v=sip(v); v.a ^= nonce; v.c ^= 187; v=sip(sip(sip(v)));
    U64 r[9]={{v.d,v.c,v.b,v.a,0,0,0,0,0}};
    v.a^=k0; v.b^=k1; v.c^=k0; v.d^=k1; v=sip(sip(sip(sip(v))));
    r[4]=v.d;r[5]=v.c;r[6]=v.b;r[7]=v.a;
    r[8]=((r[0]^r[4])&~7ULL)|3ULL;
    scratch[2048]=r[8]; unsigned cursor=2048;
")?;
    for memory in [false, true] {
        out.push_str("#pragma unroll 1\nfor (unsigned rep=0;rep<4;++rep) { unsigned budget=32;\n");
        for sub in program.programs() {
            out.push_str("for (;;) {\n");
            if !memory {
                out.push_str("cursor-=8;\n");
                for reg in 0..8 {
                    writeln!(out, "scratch[cursor+{}]=r[{reg}];", 7 - reg)?;
                }
            }
            for ins in sub.body() {
                let d = format!("r[{}]", ins.dst());
                let src = ins.src();
                let s = if memory {
                    format!("load_word(scratch,r[{src}])")
                } else {
                    format!("r[{src}]")
                };
                let imm = ins.imm();
                let expr = match ins.op() {
                    Op::MulOr => format!("({d}|{imm}ULL)*{s}"),
                    Op::MulXor => format!("({d}^{imm}ULL)*{s}"),
                    Op::MulAdd => format!("({d}+{imm}ULL)*{s}"),
                    Op::Rmcg => format!("ror({d}*r[8],{imm})"),
                    op => {
                        use Op::{
                            AddAsr, AddLsr, AddRor, SubAsr, SubLsr, SubRor, XorAsr, XorLsr, XorRor,
                        };
                        let shift = match op {
                            XorRor | AddRor | SubRor => format!("ror({d},{imm})"),
                            XorAsr | AddAsr | SubAsr => {
                                format!("U64(static_cast<long long>({d})>>{imm})")
                            }
                            XorLsr | AddLsr | SubLsr => format!("({d}>>{imm})"),
                            _ => return Err("invalid HashWX shift opcode".into()),
                        };
                        let operation = match op {
                            XorRor | XorAsr | XorLsr => "^",
                            AddRor | AddAsr | AddLsr => "+",
                            SubRor | SubAsr | SubLsr => "-",
                            _ => return Err("invalid HashWX shift opcode".into()),
                        };
                        format!("{shift}{operation}{s}")
                    }
                };
                writeln!(out, "{d}={expr};")?;
            }
            if !sub.unconditional() {
                writeln!(out, "if (r[{}]&32ULL) break;", sub.body()[0].dst())?;
            }
            out.push_str("if (budget==0) break; --budget; }\n");
        }
        out.push_str("}\n");
    }
    out.push_str(
        "
    auto lo=sip({r[0],r[1],r[2],r[3]});
    auto hi=sip({r[4],r[5],r[6],r[7]});
    return lo.d ^ hi.d ^ r[8];
}
",
    );
    out.push_str(include_str!("native_entry.cuh"));
    Ok(out)
}

fn write_words(path: &Path, words: &[u64]) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(fs::File::create(path)?);
    for &word in words {
        writer.write_all(&word.to_le_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

pub fn export(
    algorithm: Algorithm,
    seed: &[u8; 32],
    salts: &[u64],
    case: &Case,
    dest: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(dest)?;
    let source = match algorithm {
        Algorithm::HashX => hashx(&HashXProgram::generate(seed)?)?,
        Algorithm::HashWx => hashwx(&generate(seed))?,
    };
    fs::write(dest.join("kernel.cu"), source)?;
    fs::write(dest.join("kernel.wgsl"), &case.shader)?;
    write_words(&dest.join("inputs.bin"), salts)?;
    write_words(&dest.join("expected.bin"), &case.expected)?;
    Ok(())
}
