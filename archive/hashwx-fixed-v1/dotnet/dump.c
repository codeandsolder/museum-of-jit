#include <stdio.h>
#include <stdint.h>
#include "program.h"
int main(void){ uint8_t seed[32]; for(int i=0;i<32;i++)seed[i]=(uint8_t)i; siphash_key k0={0},k1={0}; for(int i=0;i<8;i++){k0.k0|=((uint64_t)seed[i])<<(8*i);k0.k1|=((uint64_t)seed[8+i])<<(8*i);k1.k0|=((uint64_t)seed[16+i])<<(8*i);k1.k1|=((uint64_t)seed[24+i])<<(8*i);} hashwx_program_list pl; hashwx_program_list_generate(&k0,&pl); printf("KEY %llu %llu\n",(unsigned long long)k1.k0,(unsigned long long)k1.k1); for(int p=0;p<32;p++){printf("P %d",p);for(int i=1;i<=8;i++){instruction*x=&pl.prog[p].code[i];printf(" %d,%u,%u,%u",(int)x->opcode,x->src,x->dst,x->imm);}printf(" %d\n",(int)pl.prog[p].code[9].opcode);}return 0;}
