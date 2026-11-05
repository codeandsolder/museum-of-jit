// Fixed HashWX, compiled into native x86-64 by LLVM 21 ORC LLJIT.
// Neither the hash nor the measured batch loop interprets bytecode at runtime.
#include "llvm/ADT/SmallVector.h"
#include "llvm/Config/llvm-config.h"
#include "llvm/ExecutionEngine/Orc/JITTargetMachineBuilder.h"
#include "llvm/ExecutionEngine/Orc/LLJIT.h"
#include "llvm/ExecutionEngine/Orc/ThreadSafeModule.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Intrinsics.h"
#include "llvm/IR/Verifier.h"
#include "llvm/InitializePasses.h"
#include "llvm/Passes/OptimizationLevel.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Support/JSON.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/raw_ostream.h"
#include <array>
#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <limits>
#include <memory>
#include <optional>
#include <sstream>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {
using Clock = std::chrono::steady_clock;
using JitHash = std::uint64_t (*)(std::uint64_t);
using JitBatch = std::uint64_t (*)(std::uint64_t, std::uint64_t);
constexpr std::uint64_t K0 = 0x1716151413121110ULL;
constexpr std::uint64_t K1 = 0x1f1e1d1c1b1a1918ULL;
constexpr std::uint64_t ExpectedCheck = 0xfd5d450b767035c5ULL;
constexpr std::size_t ScratchSize = 16392;

[[noreturn]] void fail(const std::string &why) {
  std::cerr << "LLVM_ORC_ERROR " << why << '\n';
  std::exit(EXIT_FAILURE);
}
void require(bool ok, const std::string &why) {
  if (!ok)
    fail(why);
}

struct Instruction {
  unsigned op, src, dst, imm;
};
struct Program {
  std::array<Instruction, 8> instructions;
  bool unconditional;
};
using Corpus = std::array<Program, 32>;

std::string read_text(const std::string &path) {
  std::ifstream in(path, std::ios::binary);
  require(in.good(), "cannot read corpus: " + path);
  return std::string(std::istreambuf_iterator<char>(in), {});
}

Corpus parse_corpus(const std::string &text) {
  auto parsed = llvm::json::parse(text);
  if (!parsed)
    fail("JSON parse failed: " + llvm::toString(parsed.takeError()));
  const auto *root = parsed->getAsObject();
  require(root != nullptr, "corpus must be an object");
  const auto fmt = root->getString("format");
  require(fmt && *fmt == "museum-of-jit/hashwx-fixed-program/v1",
          "bad corpus format");
  const auto *programs = root->getArray("programs");
  require(programs && programs->size() == 32, "expected exactly 32 programs");
  Corpus corpus{};
  for (std::size_t i = 0; i < corpus.size(); ++i) {
    const auto *object = (*programs)[i].getAsObject();
    require(object != nullptr, "program not object: " + std::to_string(i));
    const auto enabled = object->getBoolean("unconditional");
    require(enabled.has_value(), "missing program unconditional flag");
    corpus[i].unconditional = *enabled;
    const auto *instructions = object->getArray("instructions");
    require(instructions && instructions->size() == 8,
            "program does not have 8 instructions");
    for (std::size_t j = 0; j < 8; ++j) {
      const auto *ins = (*instructions)[j].getAsObject();
      require(ins != nullptr, "instruction not an object");
      auto op = ins->getInteger("op"), src = ins->getInteger("src");
      auto dst = ins->getInteger("dst"), imm = ins->getInteger("imm");
      require(op && src && dst && imm, "missing integer instruction field");
      require(*op >= 0 && *op <= 12 && *src >= 0 && *src <= 8 && *dst >= 0 &&
                  *dst <= 7 && *imm >= 0 && *imm < 64,
              "invalid opcode/register/immediate");
      corpus[i].instructions[j] = {
          static_cast<unsigned>(*op), static_cast<unsigned>(*src),
          static_cast<unsigned>(*dst), static_cast<unsigned>(*imm)};
    }
  }
  return corpus;
}

struct Emitter {
  llvm::LLVMContext &ctx;
  llvm::Module &mod;
  llvm::IRBuilder<> b;
  llvm::IntegerType *i64;
  llvm::Function *hash = nullptr;
  llvm::Function *rotate_left = nullptr, *rotate_right = nullptr;
  std::array<llvm::AllocaInst *, 9> regs{};
  llvm::AllocaInst *counter = nullptr, *repeat = nullptr,
                   *stack_position = nullptr;
  llvm::Value *scratch = nullptr;

  Emitter(llvm::LLVMContext &c, llvm::Module &m)
      : ctx(c), mod(m), b(c), i64(llvm::Type::getInt64Ty(c)) {
    rotate_left = llvm::Intrinsic::getOrInsertDeclaration(
        &mod, llvm::Intrinsic::fshl, {i64});
    rotate_right = llvm::Intrinsic::getOrInsertDeclaration(
        &mod, llvm::Intrinsic::fshr, {i64});
  }

  llvm::ConstantInt *num(std::uint64_t value) {
    return llvm::ConstantInt::get(i64, value);
  }
  llvm::Value *get(unsigned idx) {
    return b.CreateLoad(i64, regs[idx], "r" + std::to_string(idx));
  }
  void put(unsigned idx, llvm::Value *value) {
    b.CreateStore(value, regs[idx]);
  }
  llvm::Value *rotate(llvm::Value *v, unsigned n, bool left) {
    return b.CreateCall(left ? rotate_left : rotate_right, {v, v, num(n)});
  }
  llvm::Value *ptr_at(llvm::Value *offset) {
    // Opaque pointers; never assert `inbounds` for a data-derived address.
    return b.CreateGEP(b.getInt8Ty(), scratch, offset);
  }
  llvm::Value *ptr_at(std::uint64_t offset) { return ptr_at(num(offset)); }
  void store_unaligned(llvm::Value *at, llvm::Value *value) {
    b.CreateAlignedStore(value, at, llvm::Align(1));
  }
  llvm::Value *load_unaligned(llvm::Value *at) {
    return b.CreateAlignedLoad(i64, at, llvm::Align(1));
  }
  void sip(std::array<unsigned, 4> indices) {
    auto [a, y, c, d] = indices;
    auto *va = b.CreateAdd(get(a), get(y));
    auto *vc = b.CreateAdd(get(c), get(d));
    auto *vy = b.CreateXor(rotate(get(y), 13, true), va);
    auto *vd = b.CreateXor(rotate(get(d), 16, true), vc);
    va = b.CreateAdd(rotate(va, 32, true), vd);
    vc = b.CreateAdd(vc, vy);
    vy = b.CreateXor(rotate(vy, 17, true), vc);
    vd = b.CreateXor(rotate(vd, 21, true), va);
    put(a, va);
    put(y, vy);
    put(c, rotate(vc, 32, true));
    put(d, vd);
  }
  void snapshot() {
    auto *next = b.CreateSub(b.CreateLoad(i64, stack_position), num(64));
    b.CreateStore(next, stack_position);
    for (unsigned n = 0; n < 8; ++n) {
      auto *addr = ptr_at(b.CreateAdd(next, num(56 - 8 * n)));
      store_unaligned(addr, get(n));
    }
  }
  void instruction(const Instruction &ins, bool memory) {
    auto *lhs = get(ins.dst);
    if (ins.op == 3) {
      put(ins.dst, rotate(b.CreateMul(lhs, get(8)), ins.imm, false));
      return;
    }
    llvm::Value *stage;
    switch (ins.op) {
    case 0:
      stage = b.CreateOr(lhs, num(ins.imm));
      break;
    case 1:
      stage = b.CreateXor(lhs, num(ins.imm));
      break;
    case 2:
      stage = b.CreateAdd(lhs, num(ins.imm));
      break;
    default:
      if (ins.op <= 6)
        stage = rotate(lhs, ins.imm, false);
      else if (ins.op <= 9)
        stage = b.CreateAShr(lhs, num(ins.imm));
      else
        stage = b.CreateLShr(lhs, num(ins.imm));
    }
    // The modified destination must be visible to an aliased source register.
    put(ins.dst, stage);
    auto *source = get(ins.src);
    auto *rhs = memory ? load_unaligned(ptr_at(b.CreateAnd(source, num(16383))))
                       : source;
    llvm::Value *result;
    if (ins.op <= 2)
      result = b.CreateMul(stage, rhs);
    else if ((ins.op - 4) % 3 == 0)
      result = b.CreateXor(stage, rhs);
    else if ((ins.op - 4) % 3 == 1)
      result = b.CreateAdd(stage, rhs);
    else
      result = b.CreateSub(stage, rhs);
    put(ins.dst, result);
  }

  void program(const Program &program, bool memory) {
    auto *entry = llvm::BasicBlock::Create(ctx, "program.header", hash);
    auto *retry = llvm::BasicBlock::Create(ctx, "program.retry", hash);
    auto *done = llvm::BasicBlock::Create(ctx, "program.done", hash);
    b.CreateBr(entry);
    b.SetInsertPoint(entry);
    if (!memory)
      snapshot();
    for (const auto &ins : program.instructions)
      instruction(ins, memory);
    if (!program.unconditional) {
      auto *check_budget = llvm::BasicBlock::Create(ctx, "check.budget", hash);
      auto *value = b.CreateAnd(get(program.instructions[0].dst), num(32));
      b.CreateCondBr(b.CreateICmpNE(value, num(0)), done, check_budget);
      b.SetInsertPoint(check_budget);
    }
    auto *count = b.CreateLoad(i64, counter);
    b.CreateCondBr(b.CreateICmpNE(count, num(0)), retry, done);
    b.SetInsertPoint(retry);
    b.CreateStore(b.CreateSub(b.CreateLoad(i64, counter), num(1)), counter);
    b.CreateBr(entry);
    b.SetInsertPoint(done);
  }
  void phase(const Corpus &corpus, bool memory) {
    b.CreateStore(num(4), repeat);
    auto *header = llvm::BasicBlock::Create(ctx, "phase.header", hash);
    auto *done = llvm::BasicBlock::Create(ctx, "phase.done", hash);
    b.CreateBr(header);
    b.SetInsertPoint(header);
    b.CreateStore(num(32), counter);
    for (const auto &p : corpus)
      program(p, memory);
    auto *next = b.CreateSub(b.CreateLoad(i64, repeat), num(1));
    b.CreateStore(next, repeat);
    b.CreateCondBr(b.CreateICmpNE(next, num(0)), header, done);
    b.SetInsertPoint(done);
  }
  llvm::Function *emit_hash(const Corpus &corpus) {
    auto *type = llvm::FunctionType::get(i64, {i64}, false);
    hash = llvm::Function::Create(type, llvm::Function::ExternalLinkage,
                                  "museum_hash", mod);
    // Wasmtime's own compiled Wasm batch also makes a real call to hash.
    hash->addFnAttr(llvm::Attribute::NoInline);
    auto *entry = llvm::BasicBlock::Create(ctx, "entry", hash);
    b.SetInsertPoint(entry);
    for (unsigned i = 0; i < 9; ++i)
      regs[i] = b.CreateAlloca(i64, nullptr, "reg" + std::to_string(i));
    counter = b.CreateAlloca(i64, nullptr, "budget");
    repeat = b.CreateAlloca(i64, nullptr, "repetitions");
    stack_position = b.CreateAlloca(i64, nullptr, "scratch.position");
    scratch = b.CreateAlloca(b.getInt8Ty(), num(ScratchSize), "scratch");
    auto *salt = hash->getArg(0);
    salt->setName("salt");
    for (unsigned i = 0; i < 4; ++i) {
      constexpr std::array<std::uint64_t, 4> IV = {
          0x736f6d6570736575ULL, 0x646f72616e646f6dULL, 0x6c7967656e657261ULL,
          0x7465646279746573ULL};
      put(i, num(IV[i] ^ (i % 2 == 0 ? K0 : K1)));
    }
    put(3, b.CreateXor(get(3), salt));
    sip({0, 1, 2, 3});
    put(0, b.CreateXor(get(0), salt));
    put(2, b.CreateXor(get(2), num(0xbb)));
    for (unsigned i = 0; i < 3; ++i)
      sip({0, 1, 2, 3});
    std::array<llvm::Value *, 4> saved = {get(3), get(2), get(1), get(0)};
    for (unsigned i = 0; i < 4; ++i)
      put(i, b.CreateXor(get(i), num(i % 2 == 0 ? K0 : K1)));
    for (unsigned i = 0; i < 4; ++i)
      sip({0, 1, 2, 3});
    std::array<llvm::Value *, 4> lower = {get(3), get(2), get(1), get(0)};
    for (unsigned i = 0; i < 4; ++i)
      put(i + 4, lower[i]);
    for (unsigned i = 0; i < 4; ++i)
      put(i, saved[i]);
    auto *r8 = b.CreateOr(b.CreateAnd(b.CreateXor(get(0), get(4)), num(~7ULL)),
                          num(3));
    put(8, r8);
    store_unaligned(ptr_at(16384), r8);
    b.CreateStore(num(16384), stack_position);
    phase(corpus, false);
    phase(corpus, true);
    sip({0, 1, 2, 3});
    sip({4, 5, 6, 7});
    b.CreateRet(b.CreateXor(b.CreateXor(get(3), get(7)), get(8)));
    return hash;
  }
  llvm::Function *emit_batch() {
    auto *type = llvm::FunctionType::get(i64, {i64, i64}, false);
    auto *fn = llvm::Function::Create(type, llvm::Function::ExternalLinkage,
                                      "museum_bench", mod);
    auto it = fn->arg_begin();
    llvm::Value *start = &*it++, *count = &*it;
    auto *entry = llvm::BasicBlock::Create(ctx, "entry", fn);
    auto *loop = llvm::BasicBlock::Create(ctx, "loop", fn);
    auto *body = llvm::BasicBlock::Create(ctx, "body", fn);
    auto *done = llvm::BasicBlock::Create(ctx, "done", fn);
    b.SetInsertPoint(entry);
    b.CreateBr(loop);
    b.SetInsertPoint(loop);
    auto *index = b.CreatePHI(i64, 2, "index");
    index->addIncoming(start, entry);
    auto *remaining = b.CreatePHI(i64, 2, "remaining");
    remaining->addIncoming(count, entry);
    auto *sink = b.CreatePHI(i64, 2, "sink");
    sink->addIncoming(num(0), entry);
    b.CreateCondBr(b.CreateICmpNE(remaining, num(0)), body, done);
    b.SetInsertPoint(body);
    auto *res = b.CreateCall(hash, {index});
    auto *next_sum = b.CreateXor(sink, res);
    auto *next_index = b.CreateAdd(index, num(1));
    auto *next_count = b.CreateSub(remaining, num(1));
    index->addIncoming(next_index, body);
    remaining->addIncoming(next_count, body);
    sink->addIncoming(next_sum, body);
    b.CreateBr(loop);
    b.SetInsertPoint(done);
    b.CreateRet(sink);
    return fn;
  }
};

struct Compiled {
  std::unique_ptr<llvm::orc::LLJIT> jit;
  JitHash hash;
  JitBatch batch;
};

// IR optimization is independent of the native machine-code optimization
// level: always keep CodeGenOptLevel::Aggressive for a controlled comparison.
struct IrPolicy {
  llvm::OptimizationLevel level;
  std::string_view name;
};

IrPolicy requested_ir_policy() {
  const char *value = std::getenv("MUSEUM_LLVM_IR_OPT");
  if (value == nullptr || std::string_view(value) == "O2")
    return {llvm::OptimizationLevel::O2, "O2"};
  if (std::string_view(value) == "O0")
    return {llvm::OptimizationLevel::O0, "O0"};
  if (std::string_view(value) == "O1")
    return {llvm::OptimizationLevel::O1, "O1"};
  if (std::string_view(value) == "O3")
    return {llvm::OptimizationLevel::O3, "O3"};
  fail("MUSEUM_LLVM_IR_OPT must be exactly O0, O1, O2, or O3");
}

Compiled compile(const Corpus &corpus, IrPolicy policy) {
  llvm::InitializeNativeTarget();
  llvm::InitializeNativeTargetAsmPrinter();
  auto target = llvm::orc::JITTargetMachineBuilder::detectHost();
  if (!target)
    fail("detectHost: " + llvm::toString(target.takeError()));
  target->setCodeGenOptLevel(llvm::CodeGenOptLevel::Aggressive);
  auto jit_result = llvm::orc::LLJITBuilder()
                        .setJITTargetMachineBuilder(std::move(*target))
                        .create();
  if (!jit_result)
    fail("LLJITBuilder: " + llvm::toString(jit_result.takeError()));
  auto jit = std::move(*jit_result);
  auto ctx_owner = std::make_unique<llvm::LLVMContext>();
  auto &ctx = *ctx_owner;
  auto mod = std::make_unique<llvm::Module>("museum-hashwx-fixed-v1", ctx);
  mod->setDataLayout(jit->getDataLayout());
  mod->setTargetTriple(jit->getTargetTriple());
  Emitter emitter(ctx, *mod);
  emitter.emit_hash(corpus);
  emitter.emit_batch();
  if (llvm::verifyModule(*mod, &llvm::errs()))
    fail("LLVM IR verifier rejected generated module");

  // Unlike ORC machine codegen alone, this runs the full optimizing LLVM IR
  // pipeline, including mem2reg/loop transforms. No AOT code gets generated.
  llvm::PassBuilder pb;
  llvm::LoopAnalysisManager lam;
  llvm::FunctionAnalysisManager fam;
  llvm::CGSCCAnalysisManager cgam;
  llvm::ModuleAnalysisManager mam;
  pb.registerModuleAnalyses(mam);
  pb.registerCGSCCAnalyses(cgam);
  pb.registerFunctionAnalyses(fam);
  pb.registerLoopAnalyses(lam);
  pb.crossRegisterProxies(lam, fam, cgam, mam);
  // O0 has its own valid default pipeline; the optimized pipeline accepts O1+
  // only. Both paths still produce real native code via ORC with Aggressive
  // machine codegen. Do not use a skipped pass pipeline as an O0 shortcut.
  auto pipeline = policy.level == llvm::OptimizationLevel::O0
                      ? pb.buildO0DefaultPipeline(policy.level)
                      : pb.buildPerModuleDefaultPipeline(policy.level);
  pipeline.run(*mod, mam);
  if (llvm::verifyModule(*mod, &llvm::errs()))
    fail("optimized LLVM IR verifier rejected module");
  if (auto e = jit->addIRModule(
          llvm::orc::ThreadSafeModule(std::move(mod), std::move(ctx_owner))))
    fail("addIRModule: " + llvm::toString(std::move(e)));
  // ORC performs native code emission and relocation during these lookups.
  auto one = jit->lookup("museum_hash");
  if (!one)
    fail("lookup hash: " + llvm::toString(one.takeError()));
  auto batch = jit->lookup("museum_bench");
  if (!batch)
    fail("lookup batch: " + llvm::toString(batch.takeError()));
  return {std::move(jit), one->toPtr<JitHash>(), batch->toPtr<JitBatch>()};
}

std::uint64_t number_from_hex(std::string_view text) {
  require(text.size() == 16, "expected 16-digit unsigned hexadecimal input");
  std::uint64_t value = 0;
  for (char c : text) {
    unsigned digit;
    if (c >= '0' && c <= '9')
      digit = static_cast<unsigned>(c - '0');
    else if (c >= 'a' && c <= 'f')
      digit = static_cast<unsigned>(c - 'a' + 10);
    else if (c >= 'A' && c <= 'F')
      digit = static_cast<unsigned>(c - 'A' + 10);
    else
      fail("invalid hex input digit");
    value = (value << 4) | digit;
  }
  return value;
}
void print_hex(std::uint64_t v) {
  std::cout << std::hex << std::setw(16) << std::setfill('0') << v << std::dec
            << '\n';
}
void hash_salts(const Compiled &c, const std::string &path) {
  std::ifstream in(path);
  require(in.good(), "cannot read salts: " + path);
  std::string line;
  while (std::getline(in, line))
    print_hex(c.hash(number_from_hex(line)));
  require(in.eof(), "salts file read failure");
}
void batch_cases(const Compiled &c, const std::string &path) {
  std::ifstream in(path);
  require(in.good(), "cannot read batch cases: " + path);
  std::string line;
  while (std::getline(in, line)) {
    std::istringstream row(line);
    std::string start_text, count_text, extra;
    require(static_cast<bool>(row >> start_text >> count_text) &&
                !(row >> extra),
            "bad batch input line");
    const auto start = number_from_hex(start_text);
    const auto count = number_from_hex(count_text);
    const auto result = c.batch(start, count);
    std::uint64_t folded = 0;
    for (std::uint64_t j = 0; j < count; ++j)
      folded ^= c.hash(start + j);
    require(result == folded,
            "JIT batch/native individual-hash reduction mismatch");
    print_hex(result);
  }
  require(in.eof(), "batch cases file read failure");
}
std::uint64_t count_from_env(const char *name, std::uint64_t otherwise,
                             bool zero_ok) {
  const char *text = std::getenv(name);
  if (!text)
    return otherwise;
  std::string_view digits(text);
  require(!digits.empty(), std::string("empty ") + name);
  std::uint64_t value = 0;
  for (char c : digits) {
    require(c >= '0' && c <= '9', std::string("invalid decimal ") + name);
    const auto digit = static_cast<unsigned>(c - '0');
    require(value <= (std::numeric_limits<std::uint64_t>::max() - digit) / 10,
            std::string("overflow in ") + name);
    value = value * 10 + digit;
  }
  require(zero_ok || value > 0, std::string(name) + " must be positive");
  return value;
}
double micros(Clock::time_point t0) {
  return std::chrono::duration<double, std::micro>(Clock::now() - t0).count();
}
} // anonymous namespace

int main(int argc, char **argv) {
  const auto policy = requested_ir_policy();
  if (argc == 2 && std::string_view(argv[1]) == "--version") {
    std::cout << "LLVM ORC LLJIT " << LLVM_VERSION_STRING << " / "
              << policy.name << " IR / Aggressive codegen\n";
    return EXIT_SUCCESS;
  }
  const bool hash_file =
      (argc == 3 && std::string_view(argv[1]) == "--hash-salts");
  const bool batch_file =
      (argc == 3 && std::string_view(argv[1]) == "--batch-cases");
  require(argc == 1 || hash_file || batch_file,
          "usage: llvm-orc-hashwx [--version | --hash-salts SALTS.hex | "
          "--batch-cases CASES.hex]");
  const char *path = std::getenv("MUSEUM_PROGRAM_JSON");
  require(path && *path,
          "MUSEUM_PROGRAM_JSON must point to canonical program.json");
  const auto warmup = count_from_env("MUSEUM_WARMUP", 30000, true);
  const auto iterations = count_from_env("MUSEUM_ITERATIONS", 300000, false);
  const auto samples = count_from_env("MUSEUM_SAMPLES", 7, false);
  const auto start = Clock::now();
  const auto corpus = parse_corpus(read_text(path));
  auto compiled = compile(corpus, policy);
  const auto build_us = micros(start);
  const auto first_start = Clock::now();
  const auto check = compiled.hash(123456);
  const auto first_us = micros(first_start);
  require(check == ExpectedCheck, "independent fixture hash(123456) mismatch");
  if (hash_file) {
    hash_salts(compiled, argv[2]);
    return EXIT_SUCCESS;
  }
  if (batch_file) {
    batch_cases(compiled, argv[2]);
    return EXIT_SUCCESS;
  }
  std::cout << "ENGINE llvm-orc VERSION " << LLVM_VERSION_STRING << '\n';
  std::cout << "MODE jit-batch\n";
  std::cout << "IR_OPT " << policy.name << '\n';
  std::cout << "BUILD_US " << std::fixed << std::setprecision(3) << build_us
            << " FIRST_US " << first_us << " CHECK " << std::hex
            << std::setfill('0') << std::setw(16) << check << std::dec << '\n';
  // LLVM optimized IR includes the counted loop inside JIT code.
  volatile std::uint64_t used = compiled.batch(0, warmup);
  (void)used;
  for (std::uint64_t i = 0; i < samples; ++i) {
    const auto t0 = Clock::now();
    const auto sink = compiled.batch(0, iterations);
    const auto seconds =
        std::chrono::duration<double>(Clock::now() - t0).count();
    std::cout << "RATE " << std::fixed << std::setprecision(3)
              << static_cast<double>(iterations) / seconds
              << " sink=" << std::hex << std::setfill('0') << std::setw(16)
              << sink << std::dec << '\n';
  }
  return EXIT_SUCCESS;
}
