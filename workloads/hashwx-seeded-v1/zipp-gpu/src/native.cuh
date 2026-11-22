// Device code only: nvcc and NVRTC share precisely the same kernel.
using U64 = unsigned long long;
struct State { U64 a,b,c,d; };
__device__ __forceinline__ U64 ror(U64 value, unsigned n) {
    return (value>>n)|(value<<((64-n)&63));
}
__device__ __forceinline__ U64 umulhi(U64 a,U64 b) { return __umul64hi(a,b); }
__device__ __forceinline__ U64 smulhi(U64 a,U64 b) {
    return U64(__mul64hi(static_cast<long long>(a),static_cast<long long>(b)));
}
__device__ __forceinline__ State sip(State v) {
    v.a+=v.b;v.c+=v.d;v.b=ror(v.b,51)^v.a;v.d=ror(v.d,48)^v.c;
    v.a=ror(v.a,32);v.c+=v.b;v.a+=v.d;v.b=ror(v.b,47)^v.c;
    v.d=ror(v.d,43)^v.a;v.c=ror(v.c,32);
    return v;
}
__device__ __forceinline__ U64 load_word(const U64* scratch,U64 address) {
    unsigned pos=unsigned(address)&16383U;
    unsigned shift=(pos&7U)*8U;
    U64 low=scratch[pos>>3];
    if(!shift)return low;
    return (low>>shift)|(scratch[(pos>>3)+1]<<(64-shift));
}
