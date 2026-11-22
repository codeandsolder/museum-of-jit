// Matched-workload driver for offline cubin and runtime NVRTC specialization.
// Build with the installed CUDA toolkit and the CUDA driver API.
#include <cuda.h>
#include <nvrtc.h>
#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <vector>

using Clock = std::chrono::steady_clock;
using U64 = std::uint64_t;

static double elapsed(Clock::time_point start) {
    return std::chrono::duration<double,std::milli>(Clock::now()-start).count();
}
static void check(CUresult result) {
    if(result!=CUDA_SUCCESS) {
        const char* message=nullptr;
        cuGetErrorString(result,&message);
        throw std::runtime_error(message?message:"CUDA driver error");
    }
}
static void check(nvrtcResult result) {
    if(result!=NVRTC_SUCCESS)throw std::runtime_error(nvrtcGetErrorString(result));
}
template<class T> static std::vector<T> read(const std::string& path) {
    std::ifstream stream(path, std::ios::binary|std::ios::ate);
    if(!stream)throw std::runtime_error("cannot open "+path);
    auto bytes=stream.tellg();
    if(bytes<0 || bytes%sizeof(T))throw std::runtime_error("invalid fixture: "+path);
    std::vector<T> data(static_cast<std::size_t>(bytes)/sizeof(T));
    stream.seekg(0);
    if(!stream.read(reinterpret_cast<char*>(data.data()),bytes))
        throw std::runtime_error("cannot read "+path);
    return data;
}
struct Context {
    CUcontext value{};
    explicit Context(CUdevice device) { check(cuDevicePrimaryCtxRetain(&value,device)); check(cuCtxSetCurrent(value)); device_=device; }
    ~Context() { cuDevicePrimaryCtxRelease(device_); }
    Context(const Context&)=delete;
    Context& operator=(const Context&)=delete;
private:
    CUdevice device_{};
};
struct Allocation {
    CUdeviceptr value{};
    explicit Allocation(std::size_t bytes) {check(cuMemAlloc(&value,bytes));}
    ~Allocation(){cuMemFree(value);}
    Allocation(const Allocation&)=delete;
    Allocation& operator=(const Allocation&)=delete;
};
struct Module {
    CUmodule value{};
    ~Module(){if(value)cuModuleUnload(value);}
};
struct Event {
    CUevent value{};
    Event(){check(cuEventCreate(&value,CU_EVENT_DEFAULT));}
    ~Event(){cuEventDestroy(value);}
    Event(const Event&)=delete;
    Event& operator=(const Event&)=delete;
};
struct Program {
    nvrtcProgram value{};
    ~Program(){if(value)nvrtcDestroyProgram(&value);}
};

static std::vector<char> compile(const std::string& path,const std::string& arch) {
    auto source=read<char>(path);
    source.push_back('\0');
    Program program;
    check(nvrtcCreateProgram(&program.value,source.data(),"kernel.cu",0,nullptr,nullptr));
    const std::string architecture="--gpu-architecture="+arch;
    const char* options[]={architecture.c_str(),"--std=c++17"};
    const auto result=nvrtcCompileProgram(program.value,2,options);
    if(result!=NVRTC_SUCCESS) {
        size_t size=0;
        check(nvrtcGetProgramLogSize(program.value,&size));
        std::string log(size,'\0');
        check(nvrtcGetProgramLog(program.value,log.data()));
        std::cerr<<log<<'\n';
        check(result);
    }
    size_t size=0;
    if(arch.rfind("sm_",0)==0) {
        check(nvrtcGetCUBINSize(program.value,&size));
        std::vector<char> image(size);
        check(nvrtcGetCUBIN(program.value,image.data()));
        return image;
    }
    check(nvrtcGetPTXSize(program.value,&size));
    std::vector<char> image(size);
    check(nvrtcGetPTX(program.value,image.data()));
    return image;
}
template<class T> static void print_array(const std::vector<T>& data) {
    std::cout<<'[';
    for(std::size_t i=0;i<data.size();++i) {
        if(i)std::cout<<',';
        std::cout<<data[i];
    }
    std::cout<<']';
}
int main(int argc,char** argv) {
    try {
        if(argc!=6)throw std::runtime_error("usage: cuda-compare FIXTURE_DIR hashx|hashwx cubin|nvrtc|ptx SAMPLES BLOCK_SIZE");
        const std::string dir=argv[1],algorithm=argv[2],mode=argv[3];
        if(algorithm!="hashx"&&algorithm!="hashwx")throw std::runtime_error("unknown algorithm");
        const auto samples_arg=std::stoul(argv[4]),block_arg=std::stoul(argv[5]);
        if(samples_arg<1||samples_arg>100||block_arg<1||block_arg>1024)throw std::runtime_error("invalid dimensions");
        const auto samples=static_cast<unsigned>(samples_arg),block=static_cast<unsigned>(block_arg);
        auto inputs=read<U64>(dir+"/inputs.bin"),expected=read<U64>(dir+"/expected.bin");
        if(inputs.empty()||inputs.size()!=expected.size()||inputs.size()>std::numeric_limits<unsigned>::max())
            throw std::runtime_error("invalid input and oracle lengths");
        unsigned count=static_cast<unsigned>(inputs.size());
        check(cuInit(0));
        CUdevice device{};
        check(cuDeviceGet(&device,0));
        char name[256]{};
        check(cuDeviceGetName(name,sizeof(name),device));
        int major=0,minor=0;
        check(cuDeviceGetAttribute(&major,CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR,device));
        check(cuDeviceGetAttribute(&minor,CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR,device));
        Context context(device);
        std::string arch=std::to_string(major)+std::to_string(minor);
        Module module;
        double compile_ms=0,load_ms=0;
        if(mode=="cubin") {
            auto start=Clock::now();
            check(cuModuleLoad(&module.value,(dir+"/kernel.cubin").c_str()));
            load_ms=elapsed(start);
        } else if(mode=="nvrtc"||mode=="ptx") {
            auto start=Clock::now();
            auto image=compile(dir+"/kernel.cu",(mode=="nvrtc"?"sm_":"compute_")+arch);
            compile_ms=elapsed(start);
            start=Clock::now();
            check(cuModuleLoadData(&module.value,image.data()));
            load_ms=elapsed(start);
        } else throw std::runtime_error("unknown compile mode");
        CUfunction function{};
        auto function_start=Clock::now();
        check(cuModuleGetFunction(&function,module.value,"hash_batch"));
        load_ms+=elapsed(function_start);
        const auto bytes=inputs.size()*sizeof(U64);
        Allocation source(bytes),result(bytes),scratch(algorithm=="hashwx"?inputs.size()*2049*8:8);
        std::vector<U64> output(count);
        std::vector<double> times;
        std::vector<float> kernel_times;
        Event begin,end;
        void* arguments[]={&source.value,&result.value,&scratch.value,&count};
        for(unsigned step=0;step<=samples;++step) {
            auto start=Clock::now();
            check(cuMemcpyHtoD(source.value,inputs.data(),bytes));
            check(cuEventRecord(begin.value,nullptr));
            check(cuLaunchKernel(function,(count+block-1)/block,1,1,block,1,1,0,nullptr,arguments,nullptr));
            check(cuEventRecord(end.value,nullptr));
            check(cuMemcpyDtoH(output.data(),result.value,bytes));
            double wall_ms=elapsed(start);
            float kernel_ms=0;
            check(cuEventElapsedTime(&kernel_ms,begin.value,end.value));
            for(std::size_t i=0;i<output.size();++i)if(output[i]!=expected[i]) {
                std::cerr<<"mismatch at input "<<std::hex<<inputs[i]<<": "<<output[i]<<" != "<<expected[i]<<'\n';
                return 1;
            }
            if(step) {
                times.push_back(wall_ms);
                kernel_times.push_back(kernel_ms);
            }
        }
        int registers=0,local=0,shared=0;
        check(cuFuncGetAttribute(&registers,CU_FUNC_ATTRIBUTE_NUM_REGS,function));
        check(cuFuncGetAttribute(&local,CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES,function));
        check(cuFuncGetAttribute(&shared,CU_FUNC_ATTRIBUTE_SHARED_SIZE_BYTES,function));
        std::cout.precision(10);
        std::cout<<"{\"algorithm\":\""<<algorithm<<"\",\"implementation\":\"cuda-"<<mode
                 <<"\",\"adapter\":\""<<name<<"\",\"compute_capability\":\""<<arch
                 <<"\",\"compileMs\":"<<compile_ms<<",\"module_load_ms\":"<<load_ms
                 <<",\"registers\":"<<registers<<",\"local_bytes\":"<<local<<",\"shared_bytes\":"<<shared
                 <<",\"block_size\":"<<block<<",\"hashes_per_sample\":"<<count<<",\"verified\":"<<count
                 <<",\"times\":";
        print_array(times);
        std::cout<<",\"kernel_times_ms\":";
        print_array(kernel_times);
        std::cout<<"}\n";
    } catch(const std::exception& error) {
        std::cerr<<error.what()<<'\n';
        return 1;
    }
}
