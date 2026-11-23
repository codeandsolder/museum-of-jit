#include <cuda_runtime.h>
#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
#include "fixture.cuh"

#define CUDA_OK(call) do { \
    cudaError_t err = (call); \
    if (err != cudaSuccess) { \
        std::fprintf(stderr, "%s:%d %s: %s\n", __FILE__, __LINE__, #call, cudaGetErrorString(err)); \
        std::exit(1); \
    } \
} while (false)

#if USE_SHARED
// Exactly one 16KiB scratchpad per participating lane; no global scratch.
extern "C" __global__ void shape_kernel(const U64* inputs, U64* outputs, U64*, unsigned count) {
    extern __shared__ U64 storage[];
    unsigned idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < count) {
        Scratch scratch{storage + static_cast<U64>(threadIdx.x)*PITCH_WORDS};
        for (unsigned at = idx; at < count; at += gridDim.x * blockDim.x)
            outputs[at] = hash(inputs[at], scratch);
    }
}
#else
// Grouped interleaving: each group of GROUP_SIZE threads has word-major layout.
extern "C" __global__ void shape_kernel(const U64* inputs, U64* outputs, U64* storage, unsigned count) {
    unsigned idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < count) {
        unsigned group = idx/GROUP_SIZE, lane = idx%GROUP_SIZE;
        Scratch scratch{storage + (static_cast<U64>(group)*PITCH_WORDS*GROUP_SIZE + lane)};
        outputs[idx] = hash(inputs[idx], scratch);
    }
}
#endif

template <typename T>
std::vector<T> read_file(const char* path) {
    std::ifstream stream(path, std::ios::binary | std::ios::ate);
    if (!stream) throw std::runtime_error(std::string("failed to open ") + path);
    const auto bytes = stream.tellg();
    if (bytes < 0 || bytes % sizeof(T) != 0) throw std::runtime_error("incorrect fixture length");
    std::vector<T> result(static_cast<std::size_t>(bytes) / sizeof(T));
    stream.seekg(0);
    stream.read(reinterpret_cast<char*>(result.data()), bytes);
    if (!stream) throw std::runtime_error("failed to read fixture");
    return result;
}

int main(int argc, char** argv) {
    if (argc != 5 && argc != 6) {
        std::fprintf(stderr, "Usage: %s count block_size spoiler_shared_bytes samples\n", argv[0]);
        return 2;
    }
    const auto count = static_cast<unsigned>(std::stoul(argv[1]));
    const auto block = static_cast<unsigned>(std::stoul(argv[2]));
    const auto spoiler = static_cast<unsigned>(std::stoul(argv[3]));
    const int samples = std::stoi(argv[4]);
    if (!count || !block || !samples || count % GROUP_SIZE) throw std::runtime_error("invalid benchmark shape");
    auto inputs = read_file<U64>("inputs.bin");
    auto expected = read_file<U64>("expected.bin");
    if (inputs.size()!=4096 || expected.size()!=4096) throw std::runtime_error("unexpected museum fixture shape");
    std::vector<U64> in(count), out(count), oracle(count);
    for (unsigned i=0;i<count;++i) { in[i]=inputs[i%4096]; oracle[i]=expected[i%4096]; }
    U64 *d_in=nullptr, *d_out=nullptr, *d_scratch=nullptr;
    CUDA_OK(cudaMalloc(&d_in, count*sizeof(U64)));
    CUDA_OK(cudaMalloc(&d_out, count*sizeof(U64)));
#if !USE_SHARED
    CUDA_OK(cudaMalloc(&d_scratch, static_cast<U64>(count)*PITCH_WORDS*sizeof(U64)));
#endif
    CUDA_OK(cudaMemcpy(d_in, in.data(),count*sizeof(U64), cudaMemcpyHostToDevice));
    cudaFuncAttributes attrs{};
    CUDA_OK(cudaFuncGetAttributes(&attrs,shape_kernel));
    auto shared = USE_SHARED ? block*PITCH_WORDS*sizeof(U64) : spoiler;
    if (shared>48*1024) CUDA_OK(cudaFuncSetAttribute(shape_kernel,cudaFuncAttributeMaxDynamicSharedMemorySize,shared));
    int resident=0;
    CUDA_OK(cudaOccupancyMaxActiveBlocksPerMultiprocessor(&resident,shape_kernel,block,shared));
    int sm_count=0; CUDA_OK(cudaDeviceGetAttribute(&sm_count,cudaDevAttrMultiProcessorCount,0));
    const auto grid = argc == 6 ? static_cast<unsigned>(std::stoul(argv[5])) : (count+block-1)/block;
    std::printf("shape,group=%d,pitch=%d,shared_mode=%d,count=%u,block=%u,grid=%u,spoiler=%u,resident_ctas_sm=%d,resident_warps_sm=%.1f,regs=%d,local_bytes=%zu,shared_bytes=%u,sm_count=%d\n",GROUP_SIZE,PITCH_WORDS,USE_SHARED,count,block,grid,shared,resident,static_cast<float>(resident*((block+31)/32)),attrs.numRegs,attrs.localSizeBytes,shared,sm_count);
    auto launch = [&] {
        shape_kernel<<<grid,block,shared>>>(d_in,d_out,d_scratch,count);
        CUDA_OK(cudaPeekAtLastError());
    };
    // Warm compilation, runtime, page residency, and clocks.
    launch();launch();
    CUDA_OK(cudaDeviceSynchronize());
    CUDA_OK(cudaMemcpy(out.data(),d_out,count*sizeof(U64),cudaMemcpyDeviceToHost));
    for (unsigned i=0;i<count;++i) if (out[i]!=oracle[i]) {
        std::fprintf(stderr, "ORACLE FAIL index %u, got %016llx expected %016llx\n",i, (unsigned long long)out[i],(unsigned long long)oracle[i]);
        return 3;
    }
    cudaEvent_t begin{},end{};
    CUDA_OK(cudaEventCreate(&begin)); CUDA_OK(cudaEventCreate(&end));
    std::vector<double> times;
    for (int s=0;s<samples;++s) {
        CUDA_OK(cudaEventRecord(begin));
        launch();
        CUDA_OK(cudaEventRecord(end));
        CUDA_OK(cudaEventSynchronize(end));
        float ms=0;CUDA_OK(cudaEventElapsedTime(&ms,begin,end));times.push_back(ms);
    }
    std::sort(times.begin(),times.end());
    const double med=times[times.size()/2];
    std::printf("result,group=%d,pitch=%d,shared_mode=%d,count=%u,block=%u,spoiler=%u,samples=%d,median_ms=%.8f,rate_MHs=%.6f,verified=%u,min_ms=%.8f,max_ms=%.8f\n",
        GROUP_SIZE,PITCH_WORDS,USE_SHARED,count,block,shared,samples,med,static_cast<double>(count)/med/1000.0,count,times.front(),times.back());
    CUDA_OK(cudaEventDestroy(begin));CUDA_OK(cudaEventDestroy(end));
    CUDA_OK(cudaFree(d_in));CUDA_OK(cudaFree(d_out));if(d_scratch) CUDA_OK(cudaFree(d_scratch));
}
