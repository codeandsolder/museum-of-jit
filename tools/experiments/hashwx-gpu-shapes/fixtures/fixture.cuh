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
#ifndef GROUP_SIZE
#define GROUP_SIZE 1
#endif
#ifndef PITCH_WORDS
#define PITCH_WORDS 2049
#endif
#ifndef USE_SHARED
#define USE_SHARED 0
#endif
#ifndef HASHWX_BRANCHLESS_LOAD
#define HASHWX_BRANCHLESS_LOAD 1
#endif
struct Scratch {
    U64* data;
    __device__ __forceinline__ U64& operator[](unsigned index) const {
        return data[static_cast<U64>(index) * GROUP_SIZE];
    }
};
__device__ __forceinline__ U64 load_word(Scratch scratch,U64 address) {
    unsigned pos=unsigned(address)&16383U;
    unsigned shift=(pos&7U)*8U;
    U64 low=scratch[pos>>3];
#if HASHWX_BRANCHLESS_LOAD
    U64 high=scratch[(pos>>3)+1];
    U64 shifted=high<<((64U-shift)&63U);
    return (low>>shift)|(shifted & (U64(0)-U64(shift!=0U)));
#else
    if(!shift)return low;
    return (low>>shift)|(scratch[(pos>>3)+1]<<(64-shift));
#endif
}

#define HASHWX_SCRATCH 2049
__device__ __forceinline__ U64 hash(U64 nonce, Scratch scratch) {
    constexpr U64 k0=1663540288323457296ULL, k1=2242261671028070680ULL;
    State v{0x736f6d6570736575ULL^k0,0x646f72616e646f6dULL^k1,0x6c7967656e657261ULL^k0,0x7465646279746573ULL^k1};
    v.d ^= nonce; v=sip(v); v.a ^= nonce; v.c ^= 187; v=sip(sip(sip(v)));
    U64 r[9]={v.d,v.c,v.b,v.a,0,0,0,0,0};
    v.a^=k0; v.b^=k1; v.c^=k0; v.d^=k1; v=sip(sip(sip(sip(v))));
    r[4]=v.d;r[5]=v.c;r[6]=v.b;r[7]=v.a;
    r[8]=((r[0]^r[4])&~7ULL)|3ULL;
    scratch[2048]=r[8]; unsigned cursor=2048;

#pragma unroll 1
for (unsigned rep=0;rep<4;++rep) { unsigned budget=32;
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[7]=ror(r[7]*r[8],40);
r[6]=(r[6]>>2)-r[4];
r[5]=ror(r[5],54)-r[6];
r[3]=(r[3]>>2)+r[5];
r[2]=(r[2]^33ULL)*r[3];
r[0]=(r[0]^9ULL)*r[7];
r[1]=(r[1]|9ULL)*r[2];
r[4]=ror(r[4],60)+r[1];
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[4]=ror(r[4]*r[8],20);
r[5]=(r[5]>>2)-r[3];
r[3]=(r[3]>>3)+r[5];
r[7]=(r[7]^9ULL)*r[2];
r[0]=(r[0]^1ULL)*r[7];
r[2]=ror(r[2],45)^r[4];
r[1]=(r[1]^1ULL)*r[6];
r[6]=ror(r[6],51)-r[1];
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[0]=ror(r[0]*r[8],10);
r[2]=U64(static_cast<long long>(r[2])>>1)+r[7];
r[6]=(r[6]^33ULL)*r[4];
r[5]=(r[5]>>1)^r[3];
r[1]=(r[1]|33ULL)*r[0];
r[4]=(r[4]^9ULL)*r[6];
r[3]=ror(r[3],24)^r[1];
r[7]=U64(static_cast<long long>(r[7])>>2)-r[5];
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[5]=ror(r[5]*r[8],24);
r[1]=U64(static_cast<long long>(r[1])>>3)+r[0];
r[6]=(r[6]>>1)^r[7];
r[3]=(r[3]>>3)+r[4];
r[2]=ror(r[2],37)^r[5];
r[4]=(r[4]^33ULL)*r[3];
r[7]=(r[7]+9ULL)*r[2];
r[0]=(r[0]+1ULL)*r[1];
if (r[5]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[1]=ror(r[1]*r[8],49);
r[0]=(r[0]>>2)^r[1];
r[4]=(r[4]>>1)^r[6];
r[5]=ror(r[5],10)+r[7];
r[7]=U64(static_cast<long long>(r[7])>>2)-r[5];
r[6]=(r[6]|9ULL)*r[3];
r[3]=(r[3]|1ULL)*r[0];
r[2]=(r[2]|9ULL)*r[4];
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[3]=ror(r[3]*r[8],63);
r[5]=(r[5]^1ULL)*r[0];
r[1]=U64(static_cast<long long>(r[1])>>3)-r[4];
r[4]=(r[4]>>2)+r[1];
r[2]=(r[2]>>2)-r[3];
r[7]=(r[7]|9ULL)*r[6];
r[6]=(r[6]^33ULL)*r[7];
r[0]=(r[0]>>2)^r[2];
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[4]=ror(r[4]*r[8],59);
r[3]=(r[3]|1ULL)*r[7];
r[6]=(r[6]>>2)^r[3];
r[0]=(r[0]+9ULL)*r[6];
r[2]=ror(r[2],11)^r[0];
r[5]=U64(static_cast<long long>(r[5])>>1)-r[2];
r[7]=(r[7]^9ULL)*r[1];
r[1]=ror(r[1],5)-r[5];
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[0]=ror(r[0]*r[8],57);
r[4]=U64(static_cast<long long>(r[4])>>3)+r[7];
r[2]=(r[2]>>2)-r[4];
r[6]=(r[6]+1ULL)*r[1];
r[3]=(r[3]>>2)+r[5];
r[5]=ror(r[5],6)^r[6];
r[7]=(r[7]|1ULL)*r[3];
r[1]=(r[1]^1ULL)*r[0];
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[6]=ror(r[6]*r[8],35);
r[2]=ror(r[2],18)+r[1];
r[7]=(r[7]+1ULL)*r[6];
r[3]=(r[3]+33ULL)*r[5];
r[0]=U64(static_cast<long long>(r[0])>>3)+r[4];
r[1]=(r[1]>>3)-r[3];
r[5]=(r[5]|9ULL)*r[0];
r[4]=U64(static_cast<long long>(r[4])>>3)^r[2];
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[4]=ror(r[4]*r[8],34);
r[1]=ror(r[1],16)+r[3];
r[6]=U64(static_cast<long long>(r[6])>>1)^r[7];
r[7]=(r[7]^33ULL)*r[1];
r[2]=(r[2]^33ULL)*r[0];
r[0]=ror(r[0],32)+r[2];
r[3]=U64(static_cast<long long>(r[3])>>1)+r[6];
r[5]=(r[5]^9ULL)*r[4];
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[1]=ror(r[1]*r[8],1);
r[7]=(r[7]>>1)^r[6];
r[2]=(r[2]^1ULL)*r[3];
r[3]=(r[3]|1ULL)*r[7];
r[5]=(r[5]^33ULL)*r[4];
r[4]=(r[4]>>1)-r[5];
r[0]=ror(r[0],9)^r[1];
r[6]=U64(static_cast<long long>(r[6])>>2)^r[2];
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[3]=ror(r[3]*r[8],17);
r[0]=ror(r[0],35)+r[3];
r[5]=(r[5]>>1)^r[1];
r[4]=(r[4]|33ULL)*r[7];
r[1]=ror(r[1],7)-r[4];
r[2]=(r[2]|9ULL)*r[6];
r[7]=(r[7]^33ULL)*r[5];
r[6]=ror(r[6],55)^r[0];
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[0]=ror(r[0]*r[8],21);
r[5]=U64(static_cast<long long>(r[5])>>3)^r[6];
r[2]=(r[2]>>2)+r[5];
r[7]=(r[7]|9ULL)*r[2];
r[3]=U64(static_cast<long long>(r[3])>>1)^r[7];
r[1]=(r[1]|1ULL)*r[3];
r[6]=(r[6]|1ULL)*r[0];
r[4]=U64(static_cast<long long>(r[4])>>1)^r[1];
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[5]=ror(r[5]*r[8],41);
r[6]=ror(r[6],55)+r[3];
r[7]=(r[7]>>1)^r[6];
r[4]=(r[4]^9ULL)*r[2];
r[1]=ror(r[1],36)-r[0];
r[2]=(r[2]>>2)^r[4];
r[3]=(r[3]|1ULL)*r[5];
r[0]=(r[0]^33ULL)*r[1];
if (r[5]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[6]=ror(r[6]*r[8],19);
r[3]=ror(r[3],61)+r[6];
r[5]=(r[5]+1ULL)*r[2];
r[7]=ror(r[7],54)-r[4];
r[2]=(r[2]>>2)^r[5];
r[4]=(r[4]+33ULL)*r[7];
r[1]=(r[1]|1ULL)*r[0];
r[0]=ror(r[0],62)+r[3];
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[1]=ror(r[1]*r[8],50);
r[4]=ror(r[4],35)+r[6];
r[2]=(r[2]|1ULL)*r[5];
r[3]=(r[3]^1ULL)*r[1];
r[6]=U64(static_cast<long long>(r[6])>>1)+r[3];
r[5]=U64(static_cast<long long>(r[5])>>2)-r[2];
r[0]=(r[0]|33ULL)*r[7];
r[7]=U64(static_cast<long long>(r[7])>>2)+r[0];
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[0]=ror(r[0]*r[8],31);
r[5]=(r[5]^33ULL)*r[6];
r[7]=U64(static_cast<long long>(r[7])>>1)-r[1];
r[3]=(r[3]|9ULL)*r[0];
r[6]=ror(r[6],36)+r[2];
r[4]=ror(r[4],37)^r[5];
r[1]=(r[1]|1ULL)*r[3];
r[2]=(r[2]>>2)+r[7];
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[3]=ror(r[3]*r[8],58);
r[7]=(r[7]>>3)-r[6];
r[6]=(r[6]^33ULL)*r[7];
r[5]=(r[5]>>1)+r[2];
r[4]=(r[4]>>1)^r[0];
r[0]=(r[0]|1ULL)*r[5];
r[2]=U64(static_cast<long long>(r[2])>>1)^r[4];
r[1]=(r[1]^33ULL)*r[3];
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[5]=ror(r[5]*r[8],34);
r[7]=U64(static_cast<long long>(r[7])>>2)-r[6];
r[4]=ror(r[4],1)-r[7];
r[0]=(r[0]>>2)+r[4];
r[2]=(r[2]^9ULL)*r[0];
r[1]=(r[1]>>2)^r[5];
r[3]=(r[3]|33ULL)*r[2];
r[6]=(r[6]^33ULL)*r[3];
if (r[5]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[2]=ror(r[2]*r[8],11);
r[6]=ror(r[6],55)^r[7];
r[0]=U64(static_cast<long long>(r[0])>>3)^r[3];
r[7]=(r[7]>>1)^r[0];
r[4]=(r[4]+1ULL)*r[2];
r[3]=U64(static_cast<long long>(r[3])>>3)^r[4];
r[1]=(r[1]^9ULL)*r[5];
r[5]=(r[5]^33ULL)*r[1];
if (r[2]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[0]=ror(r[0]*r[8],24);
r[2]=(r[2]>>1)+r[4];
r[6]=(r[6]+1ULL)*r[3];
r[1]=(r[1]|1ULL)*r[7];
r[4]=(r[4]>>1)+r[2];
r[3]=(r[3]|9ULL)*r[1];
r[5]=ror(r[5],42)^r[6];
r[7]=U64(static_cast<long long>(r[7])>>3)^r[0];
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[1]=ror(r[1]*r[8],20);
r[7]=(r[7]^9ULL)*r[2];
r[5]=ror(r[5],44)-r[6];
r[2]=U64(static_cast<long long>(r[2])>>3)^r[5];
r[4]=U64(static_cast<long long>(r[4])>>2)^r[1];
r[0]=U64(static_cast<long long>(r[0])>>3)-r[3];
r[3]=(r[3]|33ULL)*r[0];
r[6]=(r[6]+9ULL)*r[4];
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[7]=ror(r[7]*r[8],22);
r[5]=U64(static_cast<long long>(r[5])>>2)+r[0];
r[6]=(r[6]>>2)+r[4];
r[2]=(r[2]^33ULL)*r[6];
r[0]=(r[0]|9ULL)*r[1];
r[3]=ror(r[3],26)^r[7];
r[4]=ror(r[4],1)+r[3];
r[1]=(r[1]^33ULL)*r[5];
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[7]=ror(r[7]*r[8],59);
r[0]=U64(static_cast<long long>(r[0])>>2)+r[1];
r[5]=(r[5]>>1)-r[2];
r[1]=(r[1]^1ULL)*r[0];
r[2]=(r[2]|9ULL)*r[3];
r[3]=(r[3]+33ULL)*r[5];
r[6]=U64(static_cast<long long>(r[6])>>2)^r[7];
r[4]=(r[4]>>2)+r[6];
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[7]=ror(r[7]*r[8],21);
r[6]=U64(static_cast<long long>(r[6])>>3)^r[3];
r[1]=(r[1]^33ULL)*r[6];
r[5]=(r[5]+9ULL)*r[1];
r[0]=(r[0]>>3)+r[7];
r[4]=ror(r[4],31)-r[5];
r[2]=(r[2]^9ULL)*r[4];
r[3]=(r[3]>>2)^r[2];
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[4]=ror(r[4]*r[8],33);
r[3]=(r[3]>>2)+r[7];
r[0]=(r[0]>>3)+r[3];
r[1]=(r[1]^1ULL)*r[5];
r[6]=U64(static_cast<long long>(r[6])>>3)-r[4];
r[7]=(r[7]|1ULL)*r[2];
r[2]=(r[2]^9ULL)*r[1];
r[5]=(r[5]>>2)+r[6];
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[3]=ror(r[3]*r[8],4);
r[4]=(r[4]>>2)+r[5];
r[5]=(r[5]>>3)+r[4];
r[7]=(r[7]^1ULL)*r[3];
r[2]=(r[2]^9ULL)*r[1];
r[1]=(r[1]|9ULL)*r[2];
r[0]=(r[0]>>2)+r[6];
r[6]=ror(r[6],22)^r[0];
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[4]=ror(r[4]*r[8],3);
r[6]=(r[6]>>1)-r[1];
r[1]=(r[1]|1ULL)*r[4];
r[0]=(r[0]|33ULL)*r[5];
r[3]=ror(r[3],19)-r[0];
r[7]=(r[7]>>3)-r[2];
r[5]=U64(static_cast<long long>(r[5])>>1)-r[6];
r[2]=(r[2]|9ULL)*r[7];
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[6]=ror(r[6]*r[8],3);
r[0]=(r[0]|9ULL)*r[3];
r[5]=(r[5]+33ULL)*r[7];
r[3]=(r[3]|9ULL)*r[2];
r[2]=(r[2]>>3)-r[0];
r[1]=(r[1]>>3)+r[5];
r[4]=U64(static_cast<long long>(r[4])>>2)-r[6];
r[7]=U64(static_cast<long long>(r[7])>>3)-r[4];
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[0]=ror(r[0]*r[8],29);
r[1]=(r[1]|9ULL)*r[0];
r[4]=(r[4]^9ULL)*r[5];
r[5]=ror(r[5],47)^r[4];
r[3]=ror(r[3],13)+r[2];
r[6]=ror(r[6],51)-r[3];
r[2]=(r[2]^33ULL)*r[7];
r[7]=ror(r[7],31)^r[1];
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[6]=ror(r[6]*r[8],52);
r[3]=ror(r[3],39)^r[7];
r[4]=ror(r[4],56)-r[3];
r[5]=U64(static_cast<long long>(r[5])>>3)-r[4];
r[2]=(r[2]|1ULL)*r[5];
r[0]=(r[0]>>3)-r[6];
r[1]=(r[1]+33ULL)*r[2];
r[7]=(r[7]|1ULL)*r[1];
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
cursor-=8;
scratch[cursor+7]=r[0];
scratch[cursor+6]=r[1];
scratch[cursor+5]=r[2];
scratch[cursor+4]=r[3];
scratch[cursor+3]=r[4];
scratch[cursor+2]=r[5];
scratch[cursor+1]=r[6];
scratch[cursor+0]=r[7];
r[7]=ror(r[7]*r[8],5);
r[6]=(r[6]>>2)^r[1];
r[4]=(r[4]|9ULL)*r[5];
r[2]=(r[2]|1ULL)*r[0];
r[5]=U64(static_cast<long long>(r[5])>>3)^r[2];
r[0]=ror(r[0],62)^r[4];
r[1]=(r[1]+1ULL)*r[6];
r[3]=U64(static_cast<long long>(r[3])>>2)-r[7];
if (budget==0) break; --budget; }
}
#pragma unroll 1
for (unsigned rep=0;rep<4;++rep) { unsigned budget=32;
for (;;) {
r[7]=ror(r[7]*r[8],40);
r[6]=(r[6]>>2)-load_word(scratch,r[4]);
r[5]=ror(r[5],54)-load_word(scratch,r[6]);
r[3]=(r[3]>>2)+load_word(scratch,r[5]);
r[2]=(r[2]^33ULL)*load_word(scratch,r[3]);
r[0]=(r[0]^9ULL)*load_word(scratch,r[7]);
r[1]=(r[1]|9ULL)*load_word(scratch,r[2]);
r[4]=ror(r[4],60)+load_word(scratch,r[1]);
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[4]=ror(r[4]*r[8],20);
r[5]=(r[5]>>2)-load_word(scratch,r[3]);
r[3]=(r[3]>>3)+load_word(scratch,r[5]);
r[7]=(r[7]^9ULL)*load_word(scratch,r[2]);
r[0]=(r[0]^1ULL)*load_word(scratch,r[7]);
r[2]=ror(r[2],45)^load_word(scratch,r[4]);
r[1]=(r[1]^1ULL)*load_word(scratch,r[6]);
r[6]=ror(r[6],51)-load_word(scratch,r[1]);
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[0]=ror(r[0]*r[8],10);
r[2]=U64(static_cast<long long>(r[2])>>1)+load_word(scratch,r[7]);
r[6]=(r[6]^33ULL)*load_word(scratch,r[4]);
r[5]=(r[5]>>1)^load_word(scratch,r[3]);
r[1]=(r[1]|33ULL)*load_word(scratch,r[0]);
r[4]=(r[4]^9ULL)*load_word(scratch,r[6]);
r[3]=ror(r[3],24)^load_word(scratch,r[1]);
r[7]=U64(static_cast<long long>(r[7])>>2)-load_word(scratch,r[5]);
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[5]=ror(r[5]*r[8],24);
r[1]=U64(static_cast<long long>(r[1])>>3)+load_word(scratch,r[0]);
r[6]=(r[6]>>1)^load_word(scratch,r[7]);
r[3]=(r[3]>>3)+load_word(scratch,r[4]);
r[2]=ror(r[2],37)^load_word(scratch,r[5]);
r[4]=(r[4]^33ULL)*load_word(scratch,r[3]);
r[7]=(r[7]+9ULL)*load_word(scratch,r[2]);
r[0]=(r[0]+1ULL)*load_word(scratch,r[1]);
if (r[5]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[1]=ror(r[1]*r[8],49);
r[0]=(r[0]>>2)^load_word(scratch,r[1]);
r[4]=(r[4]>>1)^load_word(scratch,r[6]);
r[5]=ror(r[5],10)+load_word(scratch,r[7]);
r[7]=U64(static_cast<long long>(r[7])>>2)-load_word(scratch,r[5]);
r[6]=(r[6]|9ULL)*load_word(scratch,r[3]);
r[3]=(r[3]|1ULL)*load_word(scratch,r[0]);
r[2]=(r[2]|9ULL)*load_word(scratch,r[4]);
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[3]=ror(r[3]*r[8],63);
r[5]=(r[5]^1ULL)*load_word(scratch,r[0]);
r[1]=U64(static_cast<long long>(r[1])>>3)-load_word(scratch,r[4]);
r[4]=(r[4]>>2)+load_word(scratch,r[1]);
r[2]=(r[2]>>2)-load_word(scratch,r[3]);
r[7]=(r[7]|9ULL)*load_word(scratch,r[6]);
r[6]=(r[6]^33ULL)*load_word(scratch,r[7]);
r[0]=(r[0]>>2)^load_word(scratch,r[2]);
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[4]=ror(r[4]*r[8],59);
r[3]=(r[3]|1ULL)*load_word(scratch,r[7]);
r[6]=(r[6]>>2)^load_word(scratch,r[3]);
r[0]=(r[0]+9ULL)*load_word(scratch,r[6]);
r[2]=ror(r[2],11)^load_word(scratch,r[0]);
r[5]=U64(static_cast<long long>(r[5])>>1)-load_word(scratch,r[2]);
r[7]=(r[7]^9ULL)*load_word(scratch,r[1]);
r[1]=ror(r[1],5)-load_word(scratch,r[5]);
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[0]=ror(r[0]*r[8],57);
r[4]=U64(static_cast<long long>(r[4])>>3)+load_word(scratch,r[7]);
r[2]=(r[2]>>2)-load_word(scratch,r[4]);
r[6]=(r[6]+1ULL)*load_word(scratch,r[1]);
r[3]=(r[3]>>2)+load_word(scratch,r[5]);
r[5]=ror(r[5],6)^load_word(scratch,r[6]);
r[7]=(r[7]|1ULL)*load_word(scratch,r[3]);
r[1]=(r[1]^1ULL)*load_word(scratch,r[0]);
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[6]=ror(r[6]*r[8],35);
r[2]=ror(r[2],18)+load_word(scratch,r[1]);
r[7]=(r[7]+1ULL)*load_word(scratch,r[6]);
r[3]=(r[3]+33ULL)*load_word(scratch,r[5]);
r[0]=U64(static_cast<long long>(r[0])>>3)+load_word(scratch,r[4]);
r[1]=(r[1]>>3)-load_word(scratch,r[3]);
r[5]=(r[5]|9ULL)*load_word(scratch,r[0]);
r[4]=U64(static_cast<long long>(r[4])>>3)^load_word(scratch,r[2]);
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[4]=ror(r[4]*r[8],34);
r[1]=ror(r[1],16)+load_word(scratch,r[3]);
r[6]=U64(static_cast<long long>(r[6])>>1)^load_word(scratch,r[7]);
r[7]=(r[7]^33ULL)*load_word(scratch,r[1]);
r[2]=(r[2]^33ULL)*load_word(scratch,r[0]);
r[0]=ror(r[0],32)+load_word(scratch,r[2]);
r[3]=U64(static_cast<long long>(r[3])>>1)+load_word(scratch,r[6]);
r[5]=(r[5]^9ULL)*load_word(scratch,r[4]);
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[1]=ror(r[1]*r[8],1);
r[7]=(r[7]>>1)^load_word(scratch,r[6]);
r[2]=(r[2]^1ULL)*load_word(scratch,r[3]);
r[3]=(r[3]|1ULL)*load_word(scratch,r[7]);
r[5]=(r[5]^33ULL)*load_word(scratch,r[4]);
r[4]=(r[4]>>1)-load_word(scratch,r[5]);
r[0]=ror(r[0],9)^load_word(scratch,r[1]);
r[6]=U64(static_cast<long long>(r[6])>>2)^load_word(scratch,r[2]);
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[3]=ror(r[3]*r[8],17);
r[0]=ror(r[0],35)+load_word(scratch,r[3]);
r[5]=(r[5]>>1)^load_word(scratch,r[1]);
r[4]=(r[4]|33ULL)*load_word(scratch,r[7]);
r[1]=ror(r[1],7)-load_word(scratch,r[4]);
r[2]=(r[2]|9ULL)*load_word(scratch,r[6]);
r[7]=(r[7]^33ULL)*load_word(scratch,r[5]);
r[6]=ror(r[6],55)^load_word(scratch,r[0]);
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[0]=ror(r[0]*r[8],21);
r[5]=U64(static_cast<long long>(r[5])>>3)^load_word(scratch,r[6]);
r[2]=(r[2]>>2)+load_word(scratch,r[5]);
r[7]=(r[7]|9ULL)*load_word(scratch,r[2]);
r[3]=U64(static_cast<long long>(r[3])>>1)^load_word(scratch,r[7]);
r[1]=(r[1]|1ULL)*load_word(scratch,r[3]);
r[6]=(r[6]|1ULL)*load_word(scratch,r[0]);
r[4]=U64(static_cast<long long>(r[4])>>1)^load_word(scratch,r[1]);
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[5]=ror(r[5]*r[8],41);
r[6]=ror(r[6],55)+load_word(scratch,r[3]);
r[7]=(r[7]>>1)^load_word(scratch,r[6]);
r[4]=(r[4]^9ULL)*load_word(scratch,r[2]);
r[1]=ror(r[1],36)-load_word(scratch,r[0]);
r[2]=(r[2]>>2)^load_word(scratch,r[4]);
r[3]=(r[3]|1ULL)*load_word(scratch,r[5]);
r[0]=(r[0]^33ULL)*load_word(scratch,r[1]);
if (r[5]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[6]=ror(r[6]*r[8],19);
r[3]=ror(r[3],61)+load_word(scratch,r[6]);
r[5]=(r[5]+1ULL)*load_word(scratch,r[2]);
r[7]=ror(r[7],54)-load_word(scratch,r[4]);
r[2]=(r[2]>>2)^load_word(scratch,r[5]);
r[4]=(r[4]+33ULL)*load_word(scratch,r[7]);
r[1]=(r[1]|1ULL)*load_word(scratch,r[0]);
r[0]=ror(r[0],62)+load_word(scratch,r[3]);
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[1]=ror(r[1]*r[8],50);
r[4]=ror(r[4],35)+load_word(scratch,r[6]);
r[2]=(r[2]|1ULL)*load_word(scratch,r[5]);
r[3]=(r[3]^1ULL)*load_word(scratch,r[1]);
r[6]=U64(static_cast<long long>(r[6])>>1)+load_word(scratch,r[3]);
r[5]=U64(static_cast<long long>(r[5])>>2)-load_word(scratch,r[2]);
r[0]=(r[0]|33ULL)*load_word(scratch,r[7]);
r[7]=U64(static_cast<long long>(r[7])>>2)+load_word(scratch,r[0]);
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[0]=ror(r[0]*r[8],31);
r[5]=(r[5]^33ULL)*load_word(scratch,r[6]);
r[7]=U64(static_cast<long long>(r[7])>>1)-load_word(scratch,r[1]);
r[3]=(r[3]|9ULL)*load_word(scratch,r[0]);
r[6]=ror(r[6],36)+load_word(scratch,r[2]);
r[4]=ror(r[4],37)^load_word(scratch,r[5]);
r[1]=(r[1]|1ULL)*load_word(scratch,r[3]);
r[2]=(r[2]>>2)+load_word(scratch,r[7]);
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[3]=ror(r[3]*r[8],58);
r[7]=(r[7]>>3)-load_word(scratch,r[6]);
r[6]=(r[6]^33ULL)*load_word(scratch,r[7]);
r[5]=(r[5]>>1)+load_word(scratch,r[2]);
r[4]=(r[4]>>1)^load_word(scratch,r[0]);
r[0]=(r[0]|1ULL)*load_word(scratch,r[5]);
r[2]=U64(static_cast<long long>(r[2])>>1)^load_word(scratch,r[4]);
r[1]=(r[1]^33ULL)*load_word(scratch,r[3]);
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[5]=ror(r[5]*r[8],34);
r[7]=U64(static_cast<long long>(r[7])>>2)-load_word(scratch,r[6]);
r[4]=ror(r[4],1)-load_word(scratch,r[7]);
r[0]=(r[0]>>2)+load_word(scratch,r[4]);
r[2]=(r[2]^9ULL)*load_word(scratch,r[0]);
r[1]=(r[1]>>2)^load_word(scratch,r[5]);
r[3]=(r[3]|33ULL)*load_word(scratch,r[2]);
r[6]=(r[6]^33ULL)*load_word(scratch,r[3]);
if (r[5]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[2]=ror(r[2]*r[8],11);
r[6]=ror(r[6],55)^load_word(scratch,r[7]);
r[0]=U64(static_cast<long long>(r[0])>>3)^load_word(scratch,r[3]);
r[7]=(r[7]>>1)^load_word(scratch,r[0]);
r[4]=(r[4]+1ULL)*load_word(scratch,r[2]);
r[3]=U64(static_cast<long long>(r[3])>>3)^load_word(scratch,r[4]);
r[1]=(r[1]^9ULL)*load_word(scratch,r[5]);
r[5]=(r[5]^33ULL)*load_word(scratch,r[1]);
if (r[2]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[0]=ror(r[0]*r[8],24);
r[2]=(r[2]>>1)+load_word(scratch,r[4]);
r[6]=(r[6]+1ULL)*load_word(scratch,r[3]);
r[1]=(r[1]|1ULL)*load_word(scratch,r[7]);
r[4]=(r[4]>>1)+load_word(scratch,r[2]);
r[3]=(r[3]|9ULL)*load_word(scratch,r[1]);
r[5]=ror(r[5],42)^load_word(scratch,r[6]);
r[7]=U64(static_cast<long long>(r[7])>>3)^load_word(scratch,r[0]);
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[1]=ror(r[1]*r[8],20);
r[7]=(r[7]^9ULL)*load_word(scratch,r[2]);
r[5]=ror(r[5],44)-load_word(scratch,r[6]);
r[2]=U64(static_cast<long long>(r[2])>>3)^load_word(scratch,r[5]);
r[4]=U64(static_cast<long long>(r[4])>>2)^load_word(scratch,r[1]);
r[0]=U64(static_cast<long long>(r[0])>>3)-load_word(scratch,r[3]);
r[3]=(r[3]|33ULL)*load_word(scratch,r[0]);
r[6]=(r[6]+9ULL)*load_word(scratch,r[4]);
if (r[1]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[7]=ror(r[7]*r[8],22);
r[5]=U64(static_cast<long long>(r[5])>>2)+load_word(scratch,r[0]);
r[6]=(r[6]>>2)+load_word(scratch,r[4]);
r[2]=(r[2]^33ULL)*load_word(scratch,r[6]);
r[0]=(r[0]|9ULL)*load_word(scratch,r[1]);
r[3]=ror(r[3],26)^load_word(scratch,r[7]);
r[4]=ror(r[4],1)+load_word(scratch,r[3]);
r[1]=(r[1]^33ULL)*load_word(scratch,r[5]);
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[7]=ror(r[7]*r[8],59);
r[0]=U64(static_cast<long long>(r[0])>>2)+load_word(scratch,r[1]);
r[5]=(r[5]>>1)-load_word(scratch,r[2]);
r[1]=(r[1]^1ULL)*load_word(scratch,r[0]);
r[2]=(r[2]|9ULL)*load_word(scratch,r[3]);
r[3]=(r[3]+33ULL)*load_word(scratch,r[5]);
r[6]=U64(static_cast<long long>(r[6])>>2)^load_word(scratch,r[7]);
r[4]=(r[4]>>2)+load_word(scratch,r[6]);
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[7]=ror(r[7]*r[8],21);
r[6]=U64(static_cast<long long>(r[6])>>3)^load_word(scratch,r[3]);
r[1]=(r[1]^33ULL)*load_word(scratch,r[6]);
r[5]=(r[5]+9ULL)*load_word(scratch,r[1]);
r[0]=(r[0]>>3)+load_word(scratch,r[7]);
r[4]=ror(r[4],31)-load_word(scratch,r[5]);
r[2]=(r[2]^9ULL)*load_word(scratch,r[4]);
r[3]=(r[3]>>2)^load_word(scratch,r[2]);
if (r[7]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[4]=ror(r[4]*r[8],33);
r[3]=(r[3]>>2)+load_word(scratch,r[7]);
r[0]=(r[0]>>3)+load_word(scratch,r[3]);
r[1]=(r[1]^1ULL)*load_word(scratch,r[5]);
r[6]=U64(static_cast<long long>(r[6])>>3)-load_word(scratch,r[4]);
r[7]=(r[7]|1ULL)*load_word(scratch,r[2]);
r[2]=(r[2]^9ULL)*load_word(scratch,r[1]);
r[5]=(r[5]>>2)+load_word(scratch,r[6]);
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[3]=ror(r[3]*r[8],4);
r[4]=(r[4]>>2)+load_word(scratch,r[5]);
r[5]=(r[5]>>3)+load_word(scratch,r[4]);
r[7]=(r[7]^1ULL)*load_word(scratch,r[3]);
r[2]=(r[2]^9ULL)*load_word(scratch,r[1]);
r[1]=(r[1]|9ULL)*load_word(scratch,r[2]);
r[0]=(r[0]>>2)+load_word(scratch,r[6]);
r[6]=ror(r[6],22)^load_word(scratch,r[0]);
if (r[3]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[4]=ror(r[4]*r[8],3);
r[6]=(r[6]>>1)-load_word(scratch,r[1]);
r[1]=(r[1]|1ULL)*load_word(scratch,r[4]);
r[0]=(r[0]|33ULL)*load_word(scratch,r[5]);
r[3]=ror(r[3],19)-load_word(scratch,r[0]);
r[7]=(r[7]>>3)-load_word(scratch,r[2]);
r[5]=U64(static_cast<long long>(r[5])>>1)-load_word(scratch,r[6]);
r[2]=(r[2]|9ULL)*load_word(scratch,r[7]);
if (r[4]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[6]=ror(r[6]*r[8],3);
r[0]=(r[0]|9ULL)*load_word(scratch,r[3]);
r[5]=(r[5]+33ULL)*load_word(scratch,r[7]);
r[3]=(r[3]|9ULL)*load_word(scratch,r[2]);
r[2]=(r[2]>>3)-load_word(scratch,r[0]);
r[1]=(r[1]>>3)+load_word(scratch,r[5]);
r[4]=U64(static_cast<long long>(r[4])>>2)-load_word(scratch,r[6]);
r[7]=U64(static_cast<long long>(r[7])>>3)-load_word(scratch,r[4]);
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[0]=ror(r[0]*r[8],29);
r[1]=(r[1]|9ULL)*load_word(scratch,r[0]);
r[4]=(r[4]^9ULL)*load_word(scratch,r[5]);
r[5]=ror(r[5],47)^load_word(scratch,r[4]);
r[3]=ror(r[3],13)+load_word(scratch,r[2]);
r[6]=ror(r[6],51)-load_word(scratch,r[3]);
r[2]=(r[2]^33ULL)*load_word(scratch,r[7]);
r[7]=ror(r[7],31)^load_word(scratch,r[1]);
if (r[0]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[6]=ror(r[6]*r[8],52);
r[3]=ror(r[3],39)^load_word(scratch,r[7]);
r[4]=ror(r[4],56)-load_word(scratch,r[3]);
r[5]=U64(static_cast<long long>(r[5])>>3)-load_word(scratch,r[4]);
r[2]=(r[2]|1ULL)*load_word(scratch,r[5]);
r[0]=(r[0]>>3)-load_word(scratch,r[6]);
r[1]=(r[1]+33ULL)*load_word(scratch,r[2]);
r[7]=(r[7]|1ULL)*load_word(scratch,r[1]);
if (r[6]&32ULL) break;
if (budget==0) break; --budget; }
for (;;) {
r[7]=ror(r[7]*r[8],5);
r[6]=(r[6]>>2)^load_word(scratch,r[1]);
r[4]=(r[4]|9ULL)*load_word(scratch,r[5]);
r[2]=(r[2]|1ULL)*load_word(scratch,r[0]);
r[5]=U64(static_cast<long long>(r[5])>>3)^load_word(scratch,r[2]);
r[0]=ror(r[0],62)^load_word(scratch,r[4]);
r[1]=(r[1]+1ULL)*load_word(scratch,r[6]);
r[3]=U64(static_cast<long long>(r[3])>>2)-load_word(scratch,r[7]);
if (budget==0) break; --budget; }
}

    auto lo=sip({r[0],r[1],r[2],r[3]});
    auto hi=sip({r[4],r[5],r[6],r[7]});
    return lo.d ^ hi.d ^ r[8];
}
