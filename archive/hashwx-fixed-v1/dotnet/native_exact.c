#include <hashwx.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>
static uint64_t ns(void){struct timespec t;clock_gettime(CLOCK_MONOTONIC_RAW,&t);return (uint64_t)t.tv_sec*1000000000ull+t.tv_nsec;}
int main(){uint8_t seed[32];for(int i=0;i<32;i++)seed[i]=i;hashwx_ctx*c=hashwx_alloc(HASHWX_COMPILED);hashwx_make(c,seed);printf("CHECK %016llx\n",(unsigned long long)hashwx_exec(c,123456));for(int w=0;w<30000;w++)hashwx_exec(c,w);const int N=500000;for(int s=0;s<7;s++){uint64_t z=0,t=ns();for(int i=0;i<N;i++)z^=hashwx_exec(c,i);uint64_t e=ns()-t;printf("RATE %.3f sink=%016llx\n",N*1e9/(double)e,(unsigned long long)z);}hashwx_free(c);}
