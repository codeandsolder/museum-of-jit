//! Native Cranelift emission of the full canonical fixed `HashWX` program.
//! No bytecode interpreter runs inside the measured hash loop.
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    AbiParam, InstBuilder, MemFlagsData, StackSlotData, StackSlotKind, Value, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};
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
    programs: [Program; 32],
}
#[derive(Deserialize)]
struct Program {
    instructions: [Instruction; 8],
    unconditional: bool,
}
#[derive(Deserialize)]
struct Instruction {
    op: Opcode,
    src: u8,
    dst: u8,
    imm: u8,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(try_from = "u8")]
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

impl TryFrom<u8> for Opcode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OrMultiply),
            1 => Ok(Self::XorMultiply),
            2 => Ok(Self::AddMultiply),
            3 => Ok(Self::MultiplyRotate),
            4 => Ok(Self::RotateXor),
            5 => Ok(Self::RotateAdd),
            6 => Ok(Self::RotateSub),
            7 => Ok(Self::ArithmeticXor),
            8 => Ok(Self::ArithmeticAdd),
            9 => Ok(Self::ArithmeticSub),
            10 => Ok(Self::LogicalXor),
            11 => Ok(Self::LogicalAdd),
            12 => Ok(Self::LogicalSub),
            _ => Err(format!("unsupported HashWX opcode {value}")),
        }
    }
}

fn validate(corpus: &Corpus) -> Result<(), Box<dyn Error>> {
    if corpus.format != "museum-of-jit/hashwx-fixed-program/v1" {
        return Err("unexpected corpus version".into());
    }
    // The last program must exhaust the phase-local branch budget so the
    // first phase populates every scratch byte before the second reads it.
    if !corpus.programs[31].unconditional {
        return Err("final program must be unconditional".into());
    }
    for (pidx, program) in corpus.programs.iter().enumerate() {
        for (iidx, ins) in program.instructions.iter().enumerate() {
            if ins.src > 8 || ins.dst > 7 || ins.imm >= 64 {
                return Err(format!("invalid instruction {pidx}:{iidx}").into());
            }
        }
    }
    Ok(())
}

fn constant(b: &mut FunctionBuilder<'_>, value: u64) -> Value {
    b.ins()
        .iconst(types::I64, i64::from_ne_bytes(value.to_ne_bytes()))
}
fn get(b: &mut FunctionBuilder<'_>, reg: Variable) -> Value {
    b.use_var(reg)
}
fn put(b: &mut FunctionBuilder<'_>, reg: Variable, value: Value) {
    b.def_var(reg, value);
}

fn sip(
    builder: &mut FunctionBuilder<'_>,
    regs: &[Variable; 9],
    [first, second, third, fourth]: [usize; 4],
) {
    let first_value = get(builder, regs[first]);
    let second_value = get(builder, regs[second]);
    let third_value = get(builder, regs[third]);
    let fourth_value = get(builder, regs[fourth]);
    let first_value = builder.ins().iadd(first_value, second_value);
    let third_value = builder.ins().iadd(third_value, fourth_value);
    put(builder, regs[first], first_value);
    put(builder, regs[third], third_value);
    let sh13 = constant(builder, 13);
    let sh16 = constant(builder, 16);
    let second_value = builder.ins().rotl(second_value, sh13);
    let fourth_value = builder.ins().rotl(fourth_value, sh16);
    let second_value = builder.ins().bxor(second_value, first_value);
    let fourth_value = builder.ins().bxor(fourth_value, third_value);
    put(builder, regs[second], second_value);
    put(builder, regs[fourth], fourth_value);
    let sh32 = constant(builder, 32);
    let first_value = builder.ins().rotl(first_value, sh32);
    put(builder, regs[first], first_value);
    let third_value = builder.ins().iadd(third_value, second_value);
    let first_value = builder.ins().iadd(first_value, fourth_value);
    put(builder, regs[third], third_value);
    put(builder, regs[first], first_value);
    let sh17 = constant(builder, 17);
    let sh21 = constant(builder, 21);
    let second_value = builder.ins().rotl(second_value, sh17);
    let fourth_value = builder.ins().rotl(fourth_value, sh21);
    let second_value = builder.ins().bxor(second_value, third_value);
    let fourth_value = builder.ins().bxor(fourth_value, first_value);
    put(builder, regs[second], second_value);
    put(builder, regs[fourth], fourth_value);
    let third_value = builder.ins().rotl(third_value, sh32);
    put(builder, regs[third], third_value);
}

fn snapshot(b: &mut FunctionBuilder<'_>, regs: &[Variable; 9], sp: Variable, base: Value) {
    let old = get(b, sp);
    let next = b.ins().iadd_imm_s(old, -64);
    put(b, sp, next);
    let addr = b.ins().iadd(base, next);
    for (&reg, offset) in regs[..8].iter().zip([56, 48, 40, 32, 24, 16, 8, 0]) {
        let v = get(b, reg);
        b.ins().store(MemFlagsData::new(), v, addr, offset);
    }
}

enum Combine {
    Multiply,
    Xor,
    Add,
    Sub,
}

fn instruction(
    b: &mut FunctionBuilder<'_>,
    regs: &[Variable; 9],
    base: Value,
    ins: &Instruction,
    memory: bool,
) {
    let dst = regs[usize::from(ins.dst)];
    let lhs = get(b, dst);
    let imm = i64::from(ins.imm);
    let (stage, combine) = match ins.op {
        Opcode::MultiplyRotate => {
            let r8 = get(b, regs[8]);
            let product = b.ins().imul(lhs, r8);
            let shift = constant(b, u64::from(ins.imm));
            let result = b.ins().rotr(product, shift);
            put(b, dst, result);
            return;
        }
        Opcode::OrMultiply => (b.ins().bor_imm_s(lhs, imm), Combine::Multiply),
        Opcode::XorMultiply => (b.ins().bxor_imm_s(lhs, imm), Combine::Multiply),
        Opcode::AddMultiply => (b.ins().iadd_imm_s(lhs, imm), Combine::Multiply),
        Opcode::RotateXor
        | Opcode::RotateAdd
        | Opcode::RotateSub
        | Opcode::ArithmeticXor
        | Opcode::ArithmeticAdd
        | Opcode::ArithmeticSub
        | Opcode::LogicalXor
        | Opcode::LogicalAdd
        | Opcode::LogicalSub => {
            let shift = constant(b, u64::from(ins.imm));
            let stage = match ins.op {
                Opcode::RotateXor | Opcode::RotateAdd | Opcode::RotateSub => {
                    b.ins().rotr(lhs, shift)
                }
                Opcode::ArithmeticXor | Opcode::ArithmeticAdd | Opcode::ArithmeticSub => {
                    b.ins().sshr(lhs, shift)
                }
                _ => b.ins().ushr(lhs, shift),
            };
            let combine = match ins.op {
                Opcode::RotateXor | Opcode::ArithmeticXor | Opcode::LogicalXor => Combine::Xor,
                Opcode::RotateAdd | Opcode::ArithmeticAdd | Opcode::LogicalAdd => Combine::Add,
                _ => Combine::Sub,
            };
            (stage, combine)
        }
    };
    // Update dst before reading src, including aliased registers and addresses.
    put(b, dst, stage);
    let src = get(b, regs[usize::from(ins.src)]);
    let rhs = if memory {
        let masked = b.ins().band_imm_s(src, 16_383);
        let addr = b.ins().iadd(base, masked);
        b.ins().load(types::I64, MemFlagsData::new(), addr, 0)
    } else {
        src
    };
    let result = match combine {
        Combine::Multiply => b.ins().imul(stage, rhs),
        Combine::Xor => b.ins().bxor(stage, rhs),
        Combine::Add => b.ins().iadd(stage, rhs),
        Combine::Sub => b.ins().isub(stage, rhs),
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
    // Keep the two early-exit conditions as *branches*, exactly like Wasm.
    // Materializing (budget != 0) && (branch_bit == 0) used to generate two
    // SETcc instructions per conditional program, obscuring branch lowering.
    if !p.unconditional {
        let branch_register = get(b, regs[usize::from(p.instructions[0].dst)]);
        let bit = b.ins().band_imm_s(branch_register, 32);
        let bit_set = b.ins().icmp_imm_s(IntCC::NotEqual, bit, 0);
        let check_budget = b.create_block();
        b.ins().brif(bit_set, done, &[], check_budget, &[]);
        b.switch_to_block(check_budget);
        b.seal_block(check_budget);
    }
    let count = get(b, bc);
    let has_budget = b.ins().icmp_imm_s(IntCC::NotEqual, count, 0);
    b.ins().brif(has_budget, retry, &[], done, &[]);
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
    sp: Variable,
    base: Value,
    programs: &[Program],
    memory: bool,
) {
    // Both counters are phase-local: neither escapes to the next phase.
    let bc = b.declare_var(types::I64);
    let rep = b.declare_var(types::I64);
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

// Emits the same logical batch loop as the separate Wasm bench(start,count)
// export: a 64-bit wrapping index, a remaining count, and XOR aggregation.
// The hash implementation is the exact already validated native JIT function.
fn define_batch(
    module: &mut JITModule,
    hash_id: FuncId,
) -> Result<(FuncId, usize), Box<dyn Error>> {
    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I64));
    signature.returns.push(AbiParam::new(types::I64));
    let id = module.declare_function("hashwx_batch", Linkage::Local, &signature)?;
    let mut context = module.make_context();
    context.func.signature = signature;
    let hash_ref = module.declare_func_in_func(hash_id, &mut context.func);
    let frontend_config = module.isa().frontend_config();
    let mut frontend_context = FunctionBuilderContext::new();
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut frontend_context);
        let start = b.create_block();
        b.append_block_params_for_function_params(start);
        b.switch_to_block(start);
        b.seal_block(start);
        let index = b.declare_var(types::I64);
        let remaining = b.declare_var(types::I64);
        let sink = b.declare_var(types::I64);
        let args = b.block_params(start).to_vec();
        b.def_var(index, args[0]);
        b.def_var(remaining, args[1]);
        let zero = b.ins().iconst(types::I64, 0);
        b.def_var(sink, zero);
        let header = b.create_block();
        let body = b.create_block();
        let done = b.create_block();
        b.ins().jump(header, &[]);
        b.switch_to_block(header);
        let count = b.use_var(remaining);
        let active = b.ins().icmp_imm_s(IntCC::NotEqual, count, 0);
        b.ins().brif(active, body, &[], done, &[]);
        b.switch_to_block(body);
        b.seal_block(body);
        let idx = b.use_var(index);
        let call = b.ins().call(hash_ref, &[idx]);
        let hashed = b.inst_results(call)[0];
        let acc = b.use_var(sink);
        let acc = b.ins().bxor(acc, hashed);
        b.def_var(sink, acc);
        let next_index = b.ins().iadd_imm_s(idx, 1);
        b.def_var(index, next_index);
        let current_count = b.use_var(remaining);
        let next_count = b.ins().iadd_imm_s(current_count, -1);
        b.def_var(remaining, next_count);
        b.ins().jump(header, &[]);
        b.seal_block(header);
        b.switch_to_block(done);
        b.seal_block(done);
        let result = b.use_var(sink);
        b.ins().return_(&[result]);
        b.finalize(frontend_config);
    }
    module.define_function(id, &mut context)?;
    let code_len = context
        .compiled_code()
        .ok_or("batch JIT compiled code unavailable")?
        .code_buffer()
        .len();
    module.clear_context(&mut context);
    Ok((id, code_len))
}

fn initialize_registers(b: &mut FunctionBuilder<'_>, regs: &[Variable; 9], salt: Value) -> Value {
    for (idx, value) in [
        0x736f_6d65_7073_6575 ^ K0,
        0x646f_7261_6e64_6f6d ^ K1,
        0x6c79_6765_6e65_7261 ^ K0,
        0x7465_6462_7974_6573 ^ K1,
    ]
    .into_iter()
    .enumerate()
    {
        let v = constant(b, value);
        put(b, regs[idx], v);
    }
    let v3 = get(b, regs[3]);
    let v3 = b.ins().bxor(v3, salt);
    put(b, regs[3], v3);
    sip(b, regs, [0, 1, 2, 3]);
    let v0 = get(b, regs[0]);
    let v0 = b.ins().bxor(v0, salt);
    put(b, regs[0], v0);
    let v2 = get(b, regs[2]);
    let v2 = b.ins().bxor_imm_s(v2, 0xbb);
    put(b, regs[2], v2);
    for _ in 0..3 {
        sip(b, regs, [0, 1, 2, 3]);
    }
    let saved = [
        get(b, regs[3]),
        get(b, regs[2]),
        get(b, regs[1]),
        get(b, regs[0]),
    ];
    for (idx, value) in [K0, K1, K0, K1].into_iter().enumerate() {
        let r = get(b, regs[idx]);
        let x = b
            .ins()
            .bxor_imm_s(r, i64::from_ne_bytes(value.to_ne_bytes()));
        put(b, regs[idx], x);
    }
    for _ in 0..4 {
        sip(b, regs, [0, 1, 2, 3]);
    }
    let lower = [
        get(b, regs[3]),
        get(b, regs[2]),
        get(b, regs[1]),
        get(b, regs[0]),
    ];
    for (idx, value) in lower.into_iter().enumerate() {
        put(b, regs[idx + 4], value);
    }
    for (idx, value) in saved.into_iter().enumerate() {
        put(b, regs[idx], value);
    }
    let r0 = get(b, regs[0]);
    let r4 = get(b, regs[4]);
    let x = b.ins().bxor(r0, r4);
    let x = b.ins().band_imm_s(x, -8);
    let r8 = b.ins().bor_imm_s(x, 3);
    put(b, regs[8], r8);
    r8
}

fn emit_hash(b: &mut FunctionBuilder<'_>, corpus: &Corpus, salt: Value) {
    let regs: [Variable; 9] = std::array::from_fn(|_| b.declare_var(types::I64));
    let sp = b.declare_var(types::I64);
    let scratch = b.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        SCRATCH_BYTES,
        3,
    ));
    let base = b.ins().stack_addr(types::I64, scratch, 0);
    let r8 = initialize_registers(b, &regs, salt);
    b.ins().store(MemFlagsData::new(), r8, base, 16_384);
    let sp_start = constant(b, 16_384);
    put(b, sp, sp_start);
    emit_phase(b, &regs, sp, base, &corpus.programs, false);
    emit_phase(b, &regs, sp, base, &corpus.programs, true);
    sip(b, &regs, [0, 1, 2, 3]);
    sip(b, &regs, [4, 5, 6, 7]);
    let r3 = get(b, regs[3]);
    let r7 = get(b, regs[7]);
    let r8 = get(b, regs[8]);
    let result = b.ins().bxor(r3, r7);
    let result = b.ins().bxor(result, r8);
    b.ins().return_(&[result]);
}

struct Compiled {
    // Owns the executable regions backing both native function pointers.
    _module: JITModule,
    entry: unsafe extern "C" fn(u64) -> u64,
    batch_entry: Option<unsafe extern "C" fn(u64, u64) -> u64>,
    // Compiled-code byte lengths reported by Cranelift, not guessed from pointer distances.
    hash_code_len: usize,
    batch_code_len: Option<usize>,
}

impl Compiled {
    fn build(corpus: &Corpus, batch_mode: bool) -> Result<Self, Box<dyn Error>> {
        validate(corpus)?;
        // The raw JITBuilder default is opt_level=none; use the optimizing
        // compilation policy for a meaningful native-vs-Wasm codegen comparison.
        let jit_builder =
            JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names())?;
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
            let salt = b.block_params(start)[0];
            emit_hash(&mut b, corpus, salt);
            b.finalize(frontend_config);
        }
        module.define_function(function, &mut context)?;
        let hash_code_len = context
            .compiled_code()
            .ok_or("hash JIT compiled code unavailable")?
            .code_buffer()
            .len();
        module.clear_context(&mut context);
        let (batch_id, batch_code_len) = if batch_mode {
            let (id, len) = define_batch(&mut module, function)?;
            (Some(id), Some(len))
        } else {
            (None, None)
        };
        module.finalize_definitions()?;
        let ptr = module.get_finalized_function(function);
        // SAFETY: the finalized function has the native C ABI signature (u64) -> u64
        // declared above; the module owns its executable memory.
        let entry =
            unsafe { std::mem::transmute::<*const u8, unsafe extern "C" fn(u64) -> u64>(ptr) };
        let batch_entry = batch_id.map(|id| {
            let ptr = module.get_finalized_function(id);
            // SAFETY: define_batch declared the native C ABI (u64, u64) -> u64.
            // The module owns the finalized code.
            unsafe { std::mem::transmute::<*const u8, unsafe extern "C" fn(u64, u64) -> u64>(ptr) }
        });
        Ok(Self {
            _module: module,
            entry,
            batch_entry,
            hash_code_len,
            batch_code_len,
        })
    }
    /// Write the *relocated, finalized* machine-code bytes, not a CLIF preview.
    /// Enabled only by an explicit diagnostic CLI flag; never invoked during timing.
    fn dump_code(&self, directory: &std::path::Path) -> Result<(), Box<dyn Error>> {
        std::fs::create_dir_all(directory)?;
        let hash_ptr = self.entry as *const u8;
        // SAFETY: JITModule owns finalized executable memory and `hash_code_len`
        // comes from the actual compiled code buffer, before context clearing.
        let hash = unsafe { std::slice::from_raw_parts(hash_ptr, self.hash_code_len) };
        std::fs::write(directory.join("hash.bin"), hash)?;
        if let (Some(entry), Some(len)) = (self.batch_entry, self.batch_code_len) {
            // SAFETY: same finalized JITModule ownership guarantee as the hash.
            let batch = unsafe { std::slice::from_raw_parts(entry as *const u8, len) };
            std::fs::write(directory.join("batch.bin"), batch)?;
        }
        std::fs::write(
            directory.join("manifest.txt"),
            format!(
                "engine=cranelift-native-0.137.1\nopt_level=speed\nhash_bytes={}\nbatch_bytes={}\n",
                self.hash_code_len,
                self.batch_code_len.unwrap_or(0)
            ),
        )?;
        Ok(())
    }

    #[inline]
    fn hash(&self, salt: u64) -> u64 {
        // SAFETY: build validates register bounds, immediate bounds and the
        // final unconditional program, so the generated hash initializes its
        // scratch memory before reading it. The module still owns the code.
        unsafe { (self.entry)(salt) }
    }
    #[inline]
    fn batch(&self, start: u64, count: u64) -> Result<u64, Box<dyn Error>> {
        let entry = self.batch_entry.ok_or("batch mode was not compiled")?;
        // SAFETY: this finalized native function calls the validated hash in
        // a bounded loop and has the signature declared in define_batch.
        // The module owns both code regions.
        Ok(unsafe { entry(start, count) })
    }
}

fn count(raw: &str, minimum: u32) -> Result<u32, Box<dyn Error>> {
    let value = raw.parse::<u32>()?;
    if !raw.bytes().all(|b| b.is_ascii_digit()) || !(minimum..=2_147_483_647).contains(&value) {
        return Err(format!("invalid benchmark count: {raw}").into());
    }
    Ok(value)
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

// A correctness-only interface: compare the JIT-local loop to an independent
// Rust host reduction of the same generated hash, including count=0 and wrap.
fn batch_cases(compiled: &Compiled, path: &str) -> Result<(), Box<dyn Error>> {
    let reader = std::io::BufReader::new(std::fs::File::open(path)?);
    let stdout = std::io::stdout();
    let mut output = std::io::BufWriter::new(stdout.lock());
    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 2
            || fields.iter().any(|field| {
                field.len() != 16 || !field.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
        {
            return Err(format!("invalid start/count at line {}", line_no + 1).into());
        }
        let start = u64::from_str_radix(fields[0], 16)?;
        let count = u64::from_str_radix(fields[1], 16)?;
        let jit = compiled.batch(start, count)?;
        let mut host = 0u64;
        for offset in 0..count {
            host ^= compiled.hash(start.wrapping_add(offset));
        }
        if jit != host {
            return Err(format!(
                "batch/host mismatch at line {}: {jit:016x} != {host:016x}",
                line_no + 1
            )
            .into());
        }
        writeln!(output, "{jit:016x}")?;
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
    let dump_path = match args.as_slice() {
        [mode, path] if mode == "--dump-code" => Some(std::path::Path::new(path)),
        _ => None,
    };
    let (batch_mode, oracle_input, batch_input, dimensions) = match args.as_slice() {
        [warmup, iterations, samples] => (false, None, None, Some([warmup, iterations, samples])),
        [mode, warmup, iterations, samples] if mode == "--batch" => {
            (true, None, None, Some([warmup, iterations, samples]))
        }
        [mode, path] if mode == "--hash-salts" => (false, Some(path.as_str()), None, None),
        [mode, _path] if mode == "--dump-code" => (true, None, None, None),
        [mode, path] if mode == "--batch-cases" => (true, None, Some(path.as_str()), None),
        _ => {
            return Err("usage: museum-hashwx-cranelift-direct \
            [[--batch] WARMUP ITERATIONS SAMPLES | --version | --hash-salts SALTS.hex | --batch-cases CASES.hex | --dump-code OUTPUT_DIR]"
                .into());
        }
    };
    let input = std::env::var("MUSEUM_PROGRAM_JSON")?;
    let dimensions = dimensions
        .map(|[warmup, iterations, samples]| {
            Ok::<_, Box<dyn Error>>((count(warmup, 0)?, count(iterations, 1)?, count(samples, 1)?))
        })
        .transpose()?;
    let t0 = Instant::now();
    let corpus: Corpus = serde_json::from_str(&std::fs::read_to_string(input)?)?;
    let compiled = Compiled::build(&corpus, batch_mode)?;
    let build_us = t0.elapsed().as_secs_f64() * 1e6;
    let t0 = Instant::now();
    let check = compiled.hash(123_456);
    let first_us = t0.elapsed().as_secs_f64() * 1e6;
    if check != CORRECT {
        return Err(format!("wrong result: {check:016x} != {CORRECT:016x}").into());
    }
    if let Some(path) = dump_path {
        compiled.dump_code(path)?;
        println!(
            "DUMP_CODE hash={} batch={} directory={}",
            compiled.hash_code_len,
            compiled.batch_code_len.unwrap_or(0),
            path.display()
        );
        return Ok(());
    }
    if let Some(path) = oracle_input {
        return hash_salts(&compiled, path);
    }
    if let Some(path) = batch_input {
        return batch_cases(&compiled, path);
    }
    let (warmup, iterations, samples) = dimensions.ok_or("benchmark dimensions missing")?;
    println!("MODE {}", if batch_mode { "jit-batch" } else { "direct" });
    println!("BUILD_US {build_us:.3} FIRST_US {first_us:.3} CHECK {check:016x}");
    if batch_mode {
        std::hint::black_box(compiled.batch(0, u64::from(warmup))?);
    } else {
        for n in 0..warmup {
            std::hint::black_box(compiled.hash(u64::from(n)));
        }
    }
    for _ in 0..samples {
        let t0 = Instant::now();
        let sink = if batch_mode {
            compiled.batch(0, u64::from(iterations))?
        } else {
            let mut value = 0u64;
            for n in 0..iterations {
                value ^= std::hint::black_box(compiled.hash(u64::from(n)));
            }
            value
        };
        std::hint::black_box(sink);
        let elapsed = t0.elapsed().as_secs_f64();
        println!(
            "RATE {:.3} sink={sink:016x}",
            f64::from(iterations) / elapsed
        );
    }
    Ok(())
}
