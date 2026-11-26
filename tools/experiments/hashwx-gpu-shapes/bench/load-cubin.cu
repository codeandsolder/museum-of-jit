#include <cuda.h>
#include <cuda_runtime.h>
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <stdexcept>
#include <string>
#include <vector>
#include <filesystem>

namespace fs=std::filesystem;

static void check(CUresult r,const char* f){
 if(r!=CUDA_SUCCESS){const char* m=nullptr;cuGetErrorString(r,&m);throw std::runtime_error(std::string(f)+": "+(m?m:"CUDA driver error"));}
}
static void check(cudaError_t r,const char* f){
 if(r!=cudaSuccess)throw std::runtime_error(std::string(f)+": "+cudaGetErrorString(r));
}
static std::vector<uint8_t> bytes(fs::path p){
 std::ifstream f(p,std::ios::binary|std::ios::ate);
 if(!f)throw std::runtime_error("failed open "+p.string());
 auto n=f.tellg();if(n<0)throw std::runtime_error("invalid size "+p.string());
 std::vector<uint8_t> data((size_t)n);f.seekg(0);f.read((char*)data.data(),n);
 if(!f)throw std::runtime_error("failed read "+p.string());
 return data;
}
int main(int argc,char**argv)try{
 if(argc!=3)throw std::runtime_error("Usage: load_cubin SEEDS_DIRECTORY [0|3]");
 auto root=fs::path(argv[1]);
 int samples=std::stoi(argv[2]);
 if(samples<1||samples>200)throw std::runtime_error("bad sample count");
 check(cudaFree(nullptr),"init");
 check(cuInit(0),"cuInit");
 std::vector<fs::path> dirs;
 for(auto const&entry:fs::directory_iterator(root)) if(entry.is_directory()&&fs::exists(entry.path()/"kernel.cubin"))dirs.push_back(entry.path());
 std::sort(dirs.begin(),dirs.end());
 for(auto const&seed:dirs){
   auto cubin=bytes(seed/"kernel.cubin");
   auto input=bytes(seed/"inputs.bin");
   auto golden=bytes(seed/"expected.bin");
   if(input.size()!=golden.size()|| input.size()%8 || input.empty())throw std::runtime_error("bad corpus");
   CUmodule module{}; auto before=std::chrono::steady_clock::now();
   check(cuModuleLoadData(&module,cubin.data()),"cuModuleLoadData");
   CUfunction fun{};check(cuModuleGetFunction(&fun,module,"hash_batch"),"get_function");
   double load_ms=std::chrono::duration<double,std::milli>(std::chrono::steady_clock::now()-before).count();
   int regs=0, shared=0;
   check(cuFuncGetAttribute(&regs,CU_FUNC_ATTRIBUTE_NUM_REGS,fun),"num_regs");
   check(cuFuncGetAttribute(&shared,CU_FUNC_ATTRIBUTE_SHARED_SIZE_BYTES,fun),"shared_bytes");
   printf("module,seed=%s,load_ms=%.5f,regs=%d,static_shared=%d,cubin_bytes=%zu\n",seed.filename().string().c_str(),load_ms,regs,shared,cubin.size());
   for(int count:{463,4096,65536}){
     std::vector<uint64_t> inputs(count),expected(count),outputs(count);
     auto in=(uint64_t*)input.data(),ref=(uint64_t*)golden.data();
     for(int i=0;i<count;++i){inputs[i]=in[i%(input.size()/8)];expected[i]=ref[i%(golden.size()/8)];}
     uint64_t *gpu_in{},*gpu_out{};
     check(cudaMalloc(&gpu_in,count*8),"malloc input");
     check(cudaMalloc(&gpu_out,count*8),"malloc output");
     check(cudaMemcpy(gpu_in,inputs.data(),count*8,cudaMemcpyHostToDevice),"copy input");
     CUdeviceptr in_dev=(CUdeviceptr)gpu_in,out_dev=(CUdeviceptr)gpu_out;
     unsigned count_u=count;
     void* args[]={&in_dev,&out_dev,&count_u};
     auto launch=[&](){
       check(cuLaunchKernel(fun,(count+1)/2,1,1,2,1,1,2*2049*8,nullptr,args,nullptr),"cuLaunchKernel");
     };
     launch();launch();
     check(cudaDeviceSynchronize(),"sync");
     check(cudaMemcpy(outputs.data(),gpu_out,count*8,cudaMemcpyDeviceToHost),"copy back");
     for(int i=0;i<count;++i) if(outputs[i]!=expected[i]){
       fprintf(stderr,"MISMATCH seed=%s nonceindex=%d got=%016llx expected=%016llx\n",seed.filename().string().c_str(),i,
       (unsigned long long)outputs[i],(unsigned long long)expected[i]);
       return 5;
     }
     cudaEvent_t t0{},t1{};check(cudaEventCreate(&t0),"event0");check(cudaEventCreate(&t1),"event1");
     std::vector<float> times;
     for(int s=0;s<samples;++s){
       check(cudaEventRecord(t0),"start");launch();check(cudaEventRecord(t1),"stop");check(cudaEventSynchronize(t1),"wait");
       float ms{};check(cudaEventElapsedTime(&ms,t0,t1),"elapsed");times.push_back(ms);
     }
     std::sort(times.begin(),times.end());
     double median_ms=times[times.size()/2];
     printf("verified,seed=%s,count=%d,median_ms=%.6f,mhs=%.4f\n",seed.filename().string().c_str(),count,median_ms,count/median_ms/1000.0);
     check(cudaEventDestroy(t0),"destroy0");check(cudaEventDestroy(t1),"destroy1");
     check(cudaFree(gpu_in),"free in");check(cudaFree(gpu_out),"free out");
   }
   check(cuModuleUnload(module),"unload");
 }
 return 0;
} catch(const std::exception&e){fprintf(stderr,"%s\n",e.what());return 1;}
