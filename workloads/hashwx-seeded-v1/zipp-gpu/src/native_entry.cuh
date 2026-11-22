extern "C" __global__ void hash_batch(const U64* inputs,U64* outputs,U64* scratch,unsigned count) {
    unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;
    if(idx>=count)return;
#ifdef HASHWX_SCRATCH
    outputs[idx]=hash(inputs[idx],scratch+static_cast<U64>(idx)*HASHWX_SCRATCH);
#else
    outputs[idx]=hash(inputs[idx],nullptr);
#endif
}
