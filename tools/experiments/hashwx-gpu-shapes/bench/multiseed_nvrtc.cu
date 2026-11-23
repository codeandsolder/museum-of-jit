// NVRTC compilation + CUDA-driver launch of complete seed-specialized HashWX
// CUDA sources. Each CPU-compiled seed has a 4,096-nonce reference oracle.
#include <cuda.h>
#include <cuda_runtime_api.h>
#include <nvrtc.h>
#include <algorithm>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

namespace fs = std::filesystem;

static void verify(CUresult result, const char* call) {
    if (result == CUDA_SUCCESS) return;
    const char* message = nullptr;
    cuGetErrorString(result, &message);
    throw std::runtime_error(std::string(call) + ": " + (message ? message : "CUDA error"));
}
static void verify(cudaError_t result, const char* call) {
    if (result == cudaSuccess) return;
    throw std::runtime_error(std::string(call) + ": " + cudaGetErrorString(result));
}
static void verify(nvrtcResult result, const char* call) {
    if (result == NVRTC_SUCCESS) return;
    throw std::runtime_error(std::string(call) + ": " + nvrtcGetErrorString(result));
}

template <typename T>
static std::vector<T> read_binary(const fs::path& path) {
    std::ifstream input(path, std::ios::binary | std::ios::ate);
    if (!input) throw std::runtime_error("cannot open " + path.string());
    const auto bytes = input.tellg();
    if (bytes < 0 || bytes % sizeof(T) != 0)
        throw std::runtime_error("invalid oracle length " + path.string());
    std::vector<T> values(static_cast<std::size_t>(bytes) / sizeof(T));
    input.seekg(0);
    input.read(reinterpret_cast<char*>(values.data()), bytes);
    if (!input) throw std::runtime_error("cannot read " + path.string());
    return values;
}

static std::string read_text(const fs::path& path) {
    std::ifstream input(path, std::ios::binary);
    if (!input) throw std::runtime_error("cannot open " + path.string());
    return std::string(std::istreambuf_iterator<char>(input), {});
}

using Clock = std::chrono::steady_clock;
static double elapsed_ms(Clock::time_point start) {
    return std::chrono::duration<double, std::milli>(Clock::now() - start).count();
}

constexpr unsigned THREADS_PER_BLOCK = 2;
constexpr unsigned BYTES_PER_SCRATCH = 2'049 * sizeof(unsigned long long);
constexpr unsigned DYNAMIC_SHARED_BYTES = THREADS_PER_BLOCK * BYTES_PER_SCRATCH;
constexpr auto KERNEL = R"(
extern "C" __global__ void hash_batch(const U64* inputs, U64* outputs, unsigned count) {
    extern __shared__ U64 scratchpad[];
    unsigned idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < count) {
        Scratch scratch{scratchpad + (unsigned long long)threadIdx.x * 2049ULL};
        outputs[idx] = hash(inputs[idx],scratch);
    }
}
)";

static std::vector<char> compile_seed(const fs::path& seed, double& compile_ms) {
    const auto source = read_text(seed / "fixture.cuh") + KERNEL;
    nvrtcProgram program{};
    verify(nvrtcCreateProgram(&program, source.c_str(), "hashwx-seed.cu", 0, nullptr, nullptr),
           "nvrtcCreateProgram");
    const char* opts[] = {"--gpu-architecture=sm_86", "--std=c++17", "--device-as-default-execution-space"};
    const auto t0 = Clock::now();
    const auto result = nvrtcCompileProgram(program, 3, opts);
    compile_ms = elapsed_ms(t0);
    if (result != NVRTC_SUCCESS) {
        size_t log_size = 0;
        nvrtcGetProgramLogSize(program, &log_size);
        std::string log(log_size, '\0');
        nvrtcGetProgramLog(program, log.data());
        nvrtcDestroyProgram(&program);
        throw std::runtime_error("NVRTC compile failed for " + seed.string() + ":\n" + log);
    }
    size_t size = 0;
    verify(nvrtcGetCUBINSize(program, &size), "nvrtcGetCUBINSize");
    std::vector<char> code(size);
    verify(nvrtcGetCUBIN(program, code.data()), "nvrtcGetCUBIN");
    verify(nvrtcDestroyProgram(&program), "nvrtcDestroyProgram");
    return code;
}

static void run_case(CUfunction function,
                     const std::vector<uint64_t>& oracle_inputs,
                     const std::vector<uint64_t>& oracle_expected,
                     unsigned count, unsigned samples, unsigned seed_id,
                     double compile_ms, double module_load_ms,
                     int regs, int resident_ctas) {
    std::vector<uint64_t> inputs(count), expected(count), output(count);
    for (unsigned i = 0; i < count; ++i) {
        inputs[i] = oracle_inputs[i % oracle_inputs.size()];
        expected[i] = oracle_expected[i % oracle_expected.size()];
    }
    uint64_t* d_input = nullptr;
    uint64_t* d_output = nullptr;
    verify(cudaMalloc(&d_input, count * sizeof(uint64_t)), "cudaMalloc input");
    verify(cudaMalloc(&d_output, count * sizeof(uint64_t)), "cudaMalloc output");
    verify(cudaMemcpy(d_input, inputs.data(), count * sizeof(uint64_t), cudaMemcpyHostToDevice),
           "cudaMemcpy input");
    auto in_ptr = reinterpret_cast<CUdeviceptr>(d_input);
    auto out_ptr = reinterpret_cast<CUdeviceptr>(d_output);
    void* params[] = {&in_ptr, &out_ptr, &count};
    const unsigned grid = (count + THREADS_PER_BLOCK - 1) / THREADS_PER_BLOCK;
    auto launch = [&] {
        verify(cuLaunchKernel(function, grid, 1, 1, THREADS_PER_BLOCK, 1, 1,
                              DYNAMIC_SHARED_BYTES, nullptr, params, nullptr), "cuLaunchKernel");
    };
    launch();
    launch();
    verify(cudaDeviceSynchronize(), "cudaDeviceSynchronize");
    verify(cudaMemcpy(output.data(), d_output, count * sizeof(uint64_t), cudaMemcpyDeviceToHost),
           "cudaMemcpy output");
    for (unsigned i = 0; i < count; ++i)
        if (output[i] != expected[i])
            throw std::runtime_error("incorrect HashWX hash for seed " +
                                     std::to_string(seed_id) + ", nonce " +
                                     std::to_string(inputs[i]));

    cudaEvent_t start{}, end{};
    verify(cudaEventCreate(&start), "cudaEventCreate start");
    verify(cudaEventCreate(&end), "cudaEventCreate end");
    std::vector<float> times;
    times.reserve(samples);
    for (unsigned i = 0; i < samples; ++i) {
        verify(cudaEventRecord(start), "cudaEventRecord start");
        launch();
        verify(cudaEventRecord(end), "cudaEventRecord end");
        verify(cudaEventSynchronize(end), "cudaEventSynchronize");
        float millis = 0;
        verify(cudaEventElapsedTime(&millis, start, end), "cudaEventElapsedTime");
        times.push_back(millis);
    }
    std::sort(times.begin(), times.end());
    const double median = times[times.size() / 2];
    std::printf("result,seed=%u,count=%u,compile_ms=%.3f,module_load_ms=%.3f,"
                "median_kernel_ms=%.5f,rate_MHs=%.5f,verified=%u,regs=%d,"
                "resident_ctas=%d,min_ms=%.5f,max_ms=%.5f\n",
                seed_id, count, compile_ms, module_load_ms, median, count / median / 1000.0,
                count, regs, resident_ctas, times.front(), times.back());
    std::fflush(stdout);
    verify(cudaEventDestroy(start), "cudaEventDestroy start");
    verify(cudaEventDestroy(end), "cudaEventDestroy end");
    verify(cudaFree(d_input), "cudaFree input");
    verify(cudaFree(d_output), "cudaFree output");
}

int main(int argc, char** argv) {
    try {
        if (argc != 3) throw std::runtime_error("Usage: multiseed_nvrtc SEED_ROOT SAMPLES");
        auto samples = static_cast<unsigned>(std::stoul(argv[2]));
        if (!samples) throw std::runtime_error("SAMPLES must be positive");
        verify(cudaFree(nullptr), "initialize CUDA primary context");
        verify(cuInit(0), "cuInit");
        CUcontext context{};
        verify(cuCtxGetCurrent(&context), "cuCtxGetCurrent");
        if (!context) throw std::runtime_error("no current CUDA context");

        std::vector<fs::path> directories;
        for (const auto& entry : fs::directory_iterator(argv[1]))
            if (entry.is_directory() &&
                fs::exists(entry.path() / "fixture.cuh") &&
                fs::exists(entry.path() / "expected.bin"))
                directories.push_back(entry.path());
        std::sort(directories.begin(), directories.end());
        if (directories.empty()) throw std::runtime_error("no generated seed directories");

        for (const auto& dir : directories) {
            const unsigned seed_id = static_cast<unsigned>(
                std::stoul(dir.filename().string().substr(5)));
            auto inputs = read_binary<uint64_t>(dir / "inputs.bin");
            auto expected = read_binary<uint64_t>(dir / "expected.bin");
            if (inputs.empty() || inputs.size() != expected.size() || inputs.size() < 65'536)
                throw std::runtime_error("unexpected oracle batch size");
            double compile_ms = 0;
            auto cubin = compile_seed(dir, compile_ms);
            CUmodule module{};
            const auto t0 = Clock::now();
            verify(cuModuleLoadData(&module, cubin.data()), "cuModuleLoadData");
            CUfunction function{};
            verify(cuModuleGetFunction(&function, module, "hash_batch"), "cuModuleGetFunction");
            const double load_ms = elapsed_ms(t0);
            int regs = 0;
            verify(cuFuncGetAttribute(&regs, CU_FUNC_ATTRIBUTE_NUM_REGS, function),
                   "cuFuncGetAttribute REG");
            int resident = 0;
            verify(cuOccupancyMaxActiveBlocksPerMultiprocessor(
                &resident, function, THREADS_PER_BLOCK, DYNAMIC_SHARED_BYTES),
                   "cuOccupancyMaxActiveBlocksPerMultiprocessor");
            std::printf("seed,seed=%u,cubin_bytes=%zu,compile_ms=%.3f,"
                        "module_load_ms=%.3f,regs=%d,resident_ctas=%d\n",
                        seed_id, cubin.size(), compile_ms, load_ms, regs, resident);
            for (unsigned count : {65'536U, 4096U, 463U})
                run_case(function, inputs, expected, count, samples, seed_id,
                         compile_ms, load_ms, regs, resident);
            verify(cuModuleUnload(module), "cuModuleUnload");
        }
    } catch (const std::exception& e) {
        std::fprintf(stderr, "HashWX multi-seed benchmark: %s\n", e.what());
        return 1;
    }
}
