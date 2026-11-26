//! Generate complete seed-specialized `HashWX` CUDA source and CPU-reference hashes.
//! The emitted programs are compiled using the same CUDA harness as the original fixture.

mod ptx;

use hashwx::{HashWx, Instruction, Op, ProgramSet};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

const CUDA_REFERENCE: &str = include_str!("../../fixtures/fixture.cuh");

fn seed(index: u32) -> [u8; 32] {
    let offset = index.to_le_bytes()[0].wrapping_mul(53);
    let mut result = [0_u8; 32];
    for (byte, position) in result.iter_mut().zip(0_u8..) {
        *byte = position.wrapping_add(offset);
    }
    result
}

fn emit_instruction(
    instruction: Instruction,
    memory_phase: bool,
    out: &mut String,
) -> std::fmt::Result {
    let dst = instruction.dst();
    let src = instruction.src();
    let imm = instruction.imm();
    let x = format!("r[{dst}]");
    let y = if memory_phase {
        format!("load_word(scratch,r[{src}])")
    } else {
        format!("r[{src}]")
    };
    let shift = match instruction.op() {
        Op::XorAsr | Op::AddAsr | Op::SubAsr => {
            format!("U64(static_cast<long long>({x})>>{imm})")
        }
        Op::XorLsr | Op::AddLsr | Op::SubLsr => format!("({x}>>{imm})"),
        Op::Rmcg | Op::XorRor | Op::AddRor | Op::SubRor => {
            format!("ror({x},{imm})")
        }
        Op::MulOr | Op::MulXor | Op::MulAdd => String::new(),
    };
    let value = match instruction.op() {
        Op::MulOr => format!("({x}|{imm}ULL)*{y}"),
        Op::MulXor => format!("({x}^{imm}ULL)*{y}"),
        Op::MulAdd => format!("({x}+{imm}ULL)*{y}"),
        Op::Rmcg => format!("ror({x}*r[8],{imm})"),
        Op::XorRor | Op::XorAsr | Op::XorLsr => format!("{shift}^{y}"),
        Op::AddRor | Op::AddAsr | Op::AddLsr => format!("{shift}+{y}"),
        Op::SubRor | Op::SubAsr | Op::SubLsr => format!("{shift}-{y}"),
    };
    writeln!(out, "r[{dst}]={value};")
}

fn emit_cuda(program: &ProgramSet) -> Result<String, Box<dyn std::error::Error>> {
    let (template_prefix, _) = CUDA_REFERENCE
        .split_once("for (unsigned rep=0;rep<4;++rep)")
        .ok_or("reference must contain the register phase")?;
    let template_suffix = &CUDA_REFERENCE[CUDA_REFERENCE
        .rfind("    auto lo=sip")
        .ok_or("reference must contain final SipHash")?..];
    let mut out = template_prefix.to_owned();

    let key_start = out
        .find("constexpr U64 k0=")
        .ok_or("key declaration not found")?;
    let key_end = key_start
        + out[key_start..]
            .find(';')
            .ok_or("key terminator not found")?
        + 1;
    let [key0, key1] = program.hash_key();
    out.replace_range(
        key_start..key_end,
        &format!("constexpr U64 k0={key0}ULL, k1={key1}ULL;"),
    );

    for memory_phase in [false, true] {
        out.push_str("for (unsigned rep=0;rep<4;++rep) { unsigned budget=32;\n");
        for subprogram in program.programs() {
            out.push_str("for (;;) {\n");
            if !memory_phase {
                out.push_str(
                    "cursor-=8;\n\
                     scratch[cursor+7]=r[0]; scratch[cursor+6]=r[1];\n\
                     scratch[cursor+5]=r[2]; scratch[cursor+4]=r[3];\n\
                     scratch[cursor+3]=r[4]; scratch[cursor+2]=r[5];\n\
                     scratch[cursor+1]=r[6]; scratch[cursor+0]=r[7];\n",
                );
            }
            for &instruction in subprogram.body() {
                emit_instruction(instruction, memory_phase, &mut out)?;
            }
            if !subprogram.unconditional() {
                writeln!(out, "if (r[{}]&32ULL) break;", subprogram.body()[0].dst())?;
            }
            out.push_str("if (budget==0) break; --budget;\n}\n");
        }
        out.push_str("}\n");
    }
    out.push_str(template_suffix);
    Ok(out)
}

fn emit_seed(
    directory: &Path,
    index: u32,
    nonce_count: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    let seed = seed(index);
    let t0 = Instant::now();
    let program = hashwx::generate(&seed);
    let generation_ms = t0.elapsed().as_secs_f64() * 1e3;

    let t1 = Instant::now();
    let cuda = emit_cuda(&program)?;
    let source_ms = t1.elapsed().as_secs_f64() * 1e3;
    fs::write(directory.join("fixture.cuh"), cuda)?;
    fs::write(directory.join("kernel.ptx"), ptx::emit(&program)?)?;

    let t2 = Instant::now();
    let cpu = HashWx::compile_program(&program)?;
    let cpu_compile_ms = t2.elapsed().as_secs_f64() * 1e3;
    let mut inputs = BufWriter::new(File::create(directory.join("inputs.bin"))?);
    let mut expected = BufWriter::new(File::create(directory.join("expected.bin"))?);
    for nonce in 0..nonce_count {
        inputs.write_all(&nonce.to_le_bytes())?;
        expected.write_all(&cpu.hash(nonce).to_le_bytes())?;
    }
    inputs.flush()?;
    expected.flush()?;
    let oracle_ms = t2.elapsed().as_secs_f64() * 1e3;
    println!(
        "seed_index={index},hashes={nonce_count},seed_prefix={:02x}{:02x}{:02x}{:02x},program_generation_ms={generation_ms:.4},cuda_source_generation_ms={source_ms:.4},cpu_jit_compile_ms={cpu_compile_ms:.4},cpu_oracle_total_ms={oracle_ms:.4}",
        seed[0], seed[1], seed[2], seed[3]
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let root = args
        .next()
        .ok_or("usage: generator OUTPUT_DIRECTORY SEED_COUNT NONCE_COUNT")?;
    let seeds = args.next().ok_or("missing SEED_COUNT")?.parse::<u32>()?;
    let nonce_count = args.next().ok_or("missing NONCE_COUNT")?.parse::<u64>()?;
    if seeds == 0
        || seeds > 128
        || nonce_count == 0
        || nonce_count > 1_048_576
        || args.next().is_some()
    {
        return Err("expected 1..=128 seeds and no extra arguments".into());
    }
    for index in 0..seeds {
        emit_seed(
            &Path::new(&root).join(format!("seed-{index:03}")),
            index,
            nonce_count,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_all_13_operations_in_both_phases() -> Result<(), Box<dyn std::error::Error>> {
        // All generated subprogram instructions are rendered; no fallback path exists.
        // Exercise 64 unique deterministic seeds to cover every opcode.
        let mut seen = [false; 13];
        for index in 0..64 {
            let program = hashwx::generate(&seed(index));
            let source = emit_cuda(&program)?;
            assert!(source.contains("auto lo=sip"));
            assert_eq!(source.matches("for (;;) {").count(), 64);
            for phase in 0..2 {
                for p in program.programs() {
                    for &ins in p.body() {
                        seen[ins.op() as usize] = true;
                        let mut rendered = String::new();
                        emit_instruction(ins, phase != 0, &mut rendered)?;
                        assert!(rendered.contains(&format!("r[{}]=", ins.dst())));
                    }
                }
            }
        }
        assert!(seen.into_iter().all(|value| value));
        Ok(())
    }
}
