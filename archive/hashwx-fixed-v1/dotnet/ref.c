#include <hashwx.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>
static uint64_t ns(void){struct timespec t;clock_gettime(CLOCK_MONOTONIC_RAW,&t);return (uint64_t)t.tv_sec*1000000000ull+t.tv_nsec;}
int main(){uint8_t seed[32];for(int i=0;i<32;i++)seed[i]=i;for(int typ=0;typ<2;typ++){hashwx_ctx*c=hashwx_alloc(typ?HASHWX_COMPILED:HASHWX_INTERPRETED);hashwx_make(c,seed);printf("%s CHECK %016llx\n",typ?"jit":"interp",(unsigned long long)hashwx_exec(c,123456));for(int w=0;w<30000;w++)(void)hashwx_exec(c,w);for(int s=0;s<5;s++){uint64_t sink=0,t=ns();for(uint64_t i=0;i<300000;i++)sink^=hashwx_exec(c,i);uint64_t e=ns()-t;printf("%s RATE %.3f sink=%016llx\n",typ?"jit":"interp",300000.0*1e9/e,(unsigned long long)sink);}hashwx_free(c);} }
