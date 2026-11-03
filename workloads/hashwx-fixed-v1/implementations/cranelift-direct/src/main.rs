//! Native Cranelift emission of the full canonical fixed HashWX program.
//! No bytecode interpreter runs inside the measured hash loop.
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    AbiParam, InstBuilder, MemFlagsData, StackSlotData, StackSlotKind, Value, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};
use serde::Deserialize;
use std::error::Error;
use std::io::{BufRead, Write};
use std::time::Instant;

const K0: u64 = 0x1716_1514_1312_1110;
const K1: u64 = 0x1f1e_1d1c_1b1a_1918;
const CORRECT: u64 = 0xfd5d_450b_7670_35c5;
const SCRATCH_BYTES: u32 = 16_392;

#[derive(Deserialize)]
struct Corpus {
    format: String,
    programs: Vec<Program>,
}
#[derive(Deserialize)]
struct Program {
    instructions: Vec<Instruction>,
    unconditional: bool,
}
#[derive(Deserialize)]
struct Instruction {
    op: u8,
    src: u8,
    dst: u8,
    imm: u8,
}

fn validate(corpus: &Corpus) -> Result<(), Box<dyn Error>> {
    if corpus.format != "museum-of-jit/hashwx-fixed-program/v1" || corpus.programs.len() != 32 {
        return Err("unexpected corpus version or program count".into());
    }
    for (pidx, program) in corpus.programs.iter().enumerate() {
        if program.instructions.len() != 8 {
            return Err(format!("program {pidx}: expected eight instructions").into());
        }
        for (iidx, ins) in program.instructions.iter().enumerate() {
            if ins.op > 12 || ins.src > 8 || ins.dst > 7 || ins.imm >= 64 {
                return Err(format!("invalid instruction {pidx}:{iidx}").into());
            }
        }
    }
    Ok(())
}

fn constant(b: &mut FunctionBuilder<'_>, value: u64) -> Value {
    b.ins().iconst(types::I64, value as i64)
}
fn get(b: &mut FunctionBuilder<'_>, reg: Variable) -> Value {
    b.use_var(reg)
}
fn put(b: &mut FunctionBuilder<'_>, reg: Variable, value: Value) {
    b.def_var(reg, value);
}

fn sip(b: &mut FunctionBuilder<'_>, regs: &[Variable; 9], [a, y, c, d]: [usize; 4]) {
    let va = get(b, regs[a]);
    let vy = get(b, regs[y]);
    let vc = get(b, regs[c]);
    let vd = get(b, regs[d]);
    let va = b.ins().iadd(va, vy);
    let vc = b.ins().iadd(vc, vd);
    put(b, regs[a], va);
    put(b, regs[c], vc);
    let sh13 = constant(b, 13);
    let sh16 = constant(b, 16);
    let vy = b.ins().rotl(vy, sh13);
    let vd = b.ins().rotl(vd, sh16);
    let vy = b.ins().bxor(vy, va);
    let vd = b.ins().bxor(vd, vc);
    put(b, regs[y], vy);
    put(b, regs[d], vd);
    let sh32 = constant(b, 32);
    let va = b.ins().rotl(va, sh32);
    put(b, regs[a], va);
    let vc = b.ins().iadd(vc, vy);
    let va = b.ins().iadd(va, vd);
    put(b, regs[c], vc);
    put(b, regs[a], va);
    let sh17 = constant(b, 17);
    let sh21 = constant(b, 21);
    let vy = b.ins().rotl(vy, sh17);
    let vd = b.ins().rotl(vd, sh21);
    let vy = b.ins().bxor(vy, vc);
    let vd = b.ins().bxor(vd, va);
    put(b, regs[y], vy);
    put(b, regs[d], vd);
    let vc = b.ins().rotl(vc, sh32);
    put(b, regs[c], vc);
}

fn snapshot(b: &mut FunctionBuilder<'_>, regs: &[Variable; 9], sp: Variable, base: Value) {
    let old = get(b, sp);
    let next = b.ins().iadd_imm_s(old, -64);
    put(b, sp, next);
    let addr = b.ins().iadd(base, next);
    for (idx, &reg) in regs[..8].iter().enumerate() {
        let v = get(b, reg);
        b.ins()
            .store(MemFlagsData::new(), v, addr, 56 - 8 * idx as i32);
    }
}

fn instruction(
    b: &mut FunctionBuilder<'_>,
    regs: &[Variable; 9],
    base: Value,
    ins: &Instruction,
    memory: bool,
) {
    let dst = regs[ins.dst as usize];
    let lhs = get(b, dst);
    if ins.op == 3 {
        let r8 = get(b, regs[8]);
        let product = b.ins().imul(lhs, r8);
        let n = constant(b, ins.imm as u64);
        let result = b.ins().rotr(product, n);
        put(b, dst, result);
        return;
    }
    // Exactly like the canonical Wasm emitter: update dst *before* reading src,
    // including the aliased-src-and-dst and scratch-address cases.
    let stage = match ins.op {
        0 => b.ins().bor_imm_s(lhs, ins.imm as i64),
        1 => b.ins().bxor_imm_s(lhs, ins.imm as i64),
        2 => b.ins().iadd_imm_s(lhs, ins.imm as i64),
        4..=12 => {
            let shift = constant(b, ins.imm as u64);
            match ins.op {
                4..=6 => b.ins().rotr(lhs, shift),
                7..=9 => b.ins().sshr(lhs, shift),
                _ => b.ins().ushr(lhs, shift),
            }
        }
        _ => unreachable!("validated opcode"),
    };
    put(b, dst, stage);
    let src = get(b, regs[ins.src as usize]);
    let rhs = if memory {
        let masked = b.ins().band_imm_s(src, 16_383);
        let addr = b.ins().iadd(base, masked);
        b.ins().load(types::I64, MemFlagsData::new(), addr, 0)
    } else {
        src
    };
    let result = match ins.op {
        0..=2 => b.ins().imul(stage, rhs),
        4..=12 => match (ins.op - 4) % 3 {
            0 => b.ins().bxor(stage, rhs),
            1 => b.ins().iadd(stage, rhs),
            _ => b.ins().isub(stage, rhs),
        },
        _ => unreachable!("validated opcode"),
    };
    put(b, dst, result);
}

fn emit_program(
    b: &mut FunctionBuilder<'_>,
    regs: &[Variable; 9],
    bc: Variable,
    sp: Variable,
    base: Value,
    p: &Program,
    memory: bool,
) {
    let header = b.create_block();
    let retry = b.create_block();
    let done = b.create_block();
    b.ins().jump(header, &[]);
    b.switch_to_block(header);
    if !memory {
        snapshot(b, regs, sp, base);
    }
    for ins in &p.instructions {
        instruction(b, regs, base, ins, memory);
    }
    let count = get(b, bc);
    let has_budget = b.ins().icmp_imm_s(IntCC::NotEqual, count, 0);
    let continue_loop = if p.unconditional {
        has_budget
    } else {
        let branch_register = get(b, regs[p.instructions[0].dst as usize]);
        let bit = b.ins().band_imm_s(branch_register, 32);
        let bit_clear = b.ins().icmp_imm_s(IntCC::Equal, bit, 0);
        b.ins().band(has_budget, bit_clear)
    };
    b.ins().brif(continue_loop, retry, &[], done, &[]);
    b.switch_to_block(retry);
    b.seal_block(retry);
    let count = get(b, bc);
    let dec = b.ins().iadd_imm_s(count, -1);
    put(b, bc, dec);
    b.ins().jump(header, &[]);
    b.seal_block(header);
    b.switch_to_block(done);
    b.seal_block(done);
}

fn emit_phase(
    b: &mut FunctionBuilder<'_>,
    regs: &[Variable; 9],
    bc: Variable,
    rep: Variable,
    sp: Variable,
    base: Value,
    programs: &[Program],
    memory: bool,
) {
    let four = constant(b, 4);
    put(b, rep, four);
    let header = b.create_block();
    let done = b.create_block();
    b.ins().jump(header, &[]);
    b.switch_to_block(header);
    let budget = constant(b, 32);
    put(b, bc, budget);
    for p in programs {
        emit_program(b, regs, bc, sp, base, p, memory);
    }
    let remaining = get(b, rep);
    let dec = b.ins().iadd_imm_s(remaining, -1);
    put(b, rep, dec);
    let repeat = b.ins().icmp_imm_s(IntCC::NotEqual, dec, 0);
    b.ins().brif(repeat, header, &[], done, &[]);
    b.seal_block(header);
    b.switch_to_block(done);
    b.seal_block(done);
}

struct Compiled {
    _module: JITModule,
    entry: unsafe extern "C" fn(u64) -> u64,
}

impl Compiled {
    fn build(corpus: &Corpus) -> Result<Self, Box<dyn Error>> {
        let jit_builder = JITBuilder::new(default_libcall_names())?;
        let mut module = JITModule::new(jit_builder);
        let mut signature = module.make_signature();
        signature.params.push(AbiParam::new(types::I64));
        signature.returns.push(AbiParam::new(types::I64));
        let function = module.declare_function("hashwx_fixed", Linkage::Local, &signature)?;
        let mut context = module.make_context();
        context.func.signature = signature;
        let frontend_config = module.isa().frontend_config();
        let mut frontend_context = FunctionBuilderContext::new();
        {
            let mut b = FunctionBuilder::new(&mut context.func, &mut frontend_context);
            let start = b.create_block();
            b.append_block_params_for_function_params(start);
            b.switch_to_block(start);
            b.seal_block(start);
            let regs: [Variable; 9] = std::array::from_fn(|_| b.declare_var(types::I64));
            let bc = b.declare_var(types::I64);
            let rep = b.declare_var(types::I64);
            let sp = b.declare_var(types::I64);
            let scratch = b.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                SCRATCH_BYTES,
                3,
            ));
            let base = b.ins().stack_addr(types::I64, scratch, 0);
            let salt = b.block_params(start)[0];
            for (idx, value) in [
                0x736f_6d65_7073_6575 ^ K0,
                0x646f_7261_6e64_6f6d ^ K1,
                0x6c79_6765_6e65_7261 ^ K0,
                0x7465_6462_7974_6573 ^ K1,
            ]
            .into_iter()
            .enumerate()
            {
                let v = constant(&mut b, value);
                put(&mut b, regs[idx], v);
            }
            let v3 = get(&mut b, regs[3]);
            let v3 = b.ins().bxor(v3, salt);
            put(&mut b, regs[3], v3);
            sip(&mut b, &regs, [0, 1, 2, 3]);
            let v0 = get(&mut b, regs[0]);
            let v0 = b.ins().bxor(v0, salt);
            put(&mut b, regs[0], v0);
            let v2 = get(&mut b, regs[2]);
            let v2 = b.ins().bxor_imm_s(v2, 0xbb);
            put(&mut b, regs[2], v2);
            for _ in 0..3 {
                sip(&mut b, &regs, [0, 1, 2, 3]);
            }
            let saved = [
                get(&mut b, regs[3]),
                get(&mut b, regs[2]),
                get(&mut b, regs[1]),
                get(&mut b, regs[0]),
            ];
            for (idx, value) in [K0, K1, K0, K1].into_iter().enumerate() {
                let r = get(&mut b, regs[idx]);
                let x = b.ins().bxor_imm_s(r, value as i64);
                put(&mut b, regs[idx], x);
            }
            for _ in 0..4 {
                sip(&mut b, &regs, [0, 1, 2, 3]);
            }
            let lower = [
                get(&mut b, regs[3]),
                get(&mut b, regs[2]),
                get(&mut b, regs[1]),
                get(&mut b, regs[0]),
            ];
            for (idx, value) in lower.into_iter().enumerate() {
                put(&mut b, regs[idx + 4], value);
            }
            for (idx, value) in saved.into_iter().enumerate() {
                put(&mut b, regs[idx], value);
            }
            let r0 = get(&mut b, regs[0]);
            let r4 = get(&mut b, regs[4]);
            let x = b.ins().bxor(r0, r4);
            let x = b.ins().band_imm_s(x, -8);
            let r8 = b.ins().bor_imm_s(x, 3);
            put(&mut b, regs[8], r8);
            b.ins().store(MemFlagsData::new(), r8, base, 16_384);
            let sp_start = constant(&mut b, 16_384);
            put(&mut b, sp, sp_start);
            emit_phase(&mut b, &regs, bc, rep, sp, base, &corpus.programs, false);
            emit_phase(&mut b, &regs, bc, rep, sp, base, &corpus.programs, true);
            sip(&mut b, &regs, [0, 1, 2, 3]);
            sip(&mut b, &regs, [4, 5, 6, 7]);
            let r3 = get(&mut b, regs[3]);
            let r7 = get(&mut b, regs[7]);
            let r8 = get(&mut b, regs[8]);
            let result = b.ins().bxor(r3, r7);
            let result = b.ins().bxor(result, r8);
            b.ins().return_(&[result]);
            b.finalize(frontend_config);
        }
        module.define_function(function, &mut context)?;
        module.clear_context(&mut context);
        module.finalize_definitions()?;
        let ptr = module.get_finalized_function(function);
        let entry =
            unsafe { std::mem::transmute::<*const u8, unsafe extern "C" fn(u64) -> u64>(ptr) };
        Ok(Self {
            _module: module,
            entry,
        })
    }
    #[inline]
    fn hash(&self, salt: u64) -> u64 {
        unsafe { (self.entry)(salt) }
    }
}

fn count(name: &str, default: u64, allow_zero: bool) -> Result<u64, Box<dyn Error>> {
    match std::env::var(name) {
        Ok(text) => {
            let n = text.parse::<u64>()?;
            if !allow_zero && n == 0 {
                return Err(format!("{name} must be positive").into());
            }
            Ok(n)
        }
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(e) => Err(e.into()),
    }
}

fn hash_salts(compiled: &Compiled, input_path: &str) -> Result<(), Box<dyn Error>> {
    let reader = std::io::BufReader::new(std::fs::File::open(input_path)?);
    let stdout = std::io::stdout();
    let mut output = std::io::BufWriter::new(stdout.lock());
    for (idx, line) in reader.lines().enumerate() {
        let salt = line?;
        if salt.len() != 16 || !salt.bytes().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(format!("invalid hex salt at line {}", idx + 1).into());
        }
        let input = u64::from_str_radix(&salt, 16)?;
        writeln!(output, "{:016x}", compiled.hash(input))?;
    }
    output.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--version"] {
        println!(
            "museum-hashwx-cranelift-direct {} / Cranelift 0.137.1",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    let oracle_input = match args.as_slice() {
        [] => None,
        [mode, path] if mode == "--hash-salts" => Some(path.as_str()),
        _ => {
            return Err(
                "usage: museum-hashwx-cranelift-direct [--version | --hash-salts SALTS.HEX]".into(),
            );
        }
    };
    let input = std::env::var("MUSEUM_PROGRAM_JSON")?;
    let warmup = count("MUSEUM_WARMUP", 30_000, true)?;
    let iterations = count("MUSEUM_ITERATIONS", 300_000, false)?;
    let samples = count("MUSEUM_SAMPLES", 7, false)?;
    let t0 = Instant::now();
    let corpus: Corpus = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    validate(&corpus)?;
    let compiled = Compiled::build(&corpus)?;
    let build_us = t0.elapsed().as_secs_f64() * 1e6;
    let t0 = Instant::now();
    let check = compiled.hash(123_456);
    let first_us = t0.elapsed().as_secs_f64() * 1e6;
    if check != CORRECT {
        return Err(format!("wrong result: {check:016x} != {CORRECT:016x}").into());
    }
    if let Some(path) = oracle_input {
        return hash_salts(&compiled, path);
    }
    println!("BUILD_US {build_us:.3} FIRST_US {first_us:.3} CHECK {check:016x}");
    for n in 0..warmup {
        std::hint::black_box(compiled.hash(n));
    }
    for _ in 0..samples {
        let t0 = Instant::now();
        let mut sink = 0u64;
        for n in 0..iterations {
            sink ^= std::hint::black_box(compiled.hash(n));
        }
        let elapsed = t0.elapsed().as_secs_f64();
        println!("RATE {:.3} sink={sink:016x}", iterations as f64 / elapsed);
    }
    Ok(())
}
