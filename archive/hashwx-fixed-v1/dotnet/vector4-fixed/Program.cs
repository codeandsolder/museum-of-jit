using System;
using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.Intrinsics;
using System.Runtime.Intrinsics.X86;
static class HW {
 const ulong K0=1663540288323457296UL,K1=2242261671028070680UL;
 static readonly Vector256<ulong> ZERO=Vector256<ulong>.Zero, ONE=Vector256.Create(1UL), ALL=Vector256.Create(ulong.MaxValue), BASE=Vector256.Create(0UL,16392UL,32784UL,49176UL), MASK=Vector256.Create(16383UL);
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static Vector256<ulong> V(ulong x)=>Vector256.Create(x);
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static Vector256<ulong> Sel(Vector256<ulong> m,Vector256<ulong>a,Vector256<ulong>b)=>Vector256.ConditionalSelect(m,a,b);
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static Vector256<ulong> Ror(Vector256<ulong>x,int n)=>Vector256.BitwiseOr(Vector256.ShiftRightLogical(x,n),Vector256.ShiftLeft(x,64-n));
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static Vector256<ulong> NotZero(Vector256<ulong>x)=>Vector256.Xor(Vector256.Equals(x,ZERO),ALL);
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static bool None(Vector256<ulong>x)=>(x[0]|x[1]|x[2]|x[3])==0;
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static unsafe Vector256<ulong> Load(nint mem,Vector256<ulong>a){var idx=Vector256.Add(BASE,Vector256.BitwiseAnd(a,MASK)).AsInt64();return Avx2.GatherVector256((ulong*)mem,idx,1);}
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static unsafe void Store(nint mem,Vector256<ulong>active,ref Vector256<ulong>sp,Vector256<ulong>r0,Vector256<ulong>r1,Vector256<ulong>r2,Vector256<ulong>r3,Vector256<ulong>r4,Vector256<ulong>r5,Vector256<ulong>r6,Vector256<ulong>r7){var nsp=Vector256.Subtract(sp,V(64));sp=Sel(active,nsp,sp);for(int l=0;l<4;l++){if(active[l]==0)continue;byte* p=(byte*)mem+l*16392+(nint)sp[l];Unsafe.WriteUnaligned(p+56,r0[l]);Unsafe.WriteUnaligned(p+48,r1[l]);Unsafe.WriteUnaligned(p+40,r2[l]);Unsafe.WriteUnaligned(p+32,r3[l]);Unsafe.WriteUnaligned(p+24,r4[l]);Unsafe.WriteUnaligned(p+16,r5[l]);Unsafe.WriteUnaligned(p+8,r6[l]);Unsafe.WriteUnaligned(p,r7[l]);}}
 [MethodImpl(MethodImplOptions.AggressiveInlining)] static void Sip(ref Vector256<ulong>v0,ref Vector256<ulong>v1,ref Vector256<ulong>v2,ref Vector256<ulong>v3){v0=Vector256.Add(v0,v1);v2=Vector256.Add(v2,v3);v1=Ror(v1,64-13);v3=Ror(v3,64-16);v1=Vector256.Xor(v1,v0);v3=Vector256.Xor(v3,v2);v0=Ror(v0,64-32);v2=Vector256.Add(v2,v1);v0=Vector256.Add(v0,v3);v1=Ror(v1,64-17);v3=Ror(v3,64-21);v1=Vector256.Xor(v1,v2);v3=Vector256.Xor(v3,v0);v2=Ror(v2,64-32);}
 [MethodImpl(MethodImplOptions.NoInlining)] public static unsafe Vector256<ulong> Hash4(ulong input,nint mem){
 var salt=Vector256.Create(input,input+1,input+2,input+3);var k0=V(K0);var k1=V(K1);var v0=Vector256.Xor(V(0x736f6d6570736575),k0);var v1=Vector256.Xor(V(0x646f72616e646f6d),k1);var v2=Vector256.Xor(V(0x6c7967656e657261),k0);var v3=Vector256.Xor(V(0x7465646279746573),k1);v3=Vector256.Xor(v3,salt);Sip(ref v0,ref v1,ref v2,ref v3);v0=Vector256.Xor(v0,salt);v2=Vector256.Xor(v2,V(0xbb));Sip(ref v0,ref v1,ref v2,ref v3);Sip(ref v0,ref v1,ref v2,ref v3);Sip(ref v0,ref v1,ref v2,ref v3);var r0=v3;var r1=v2;var r2=v1;var r3=v0;v0=Vector256.Xor(v0,k0);v1=Vector256.Xor(v1,k1);v2=Vector256.Xor(v2,k0);v3=Vector256.Xor(v3,k1);Sip(ref v0,ref v1,ref v2,ref v3);Sip(ref v0,ref v1,ref v2,ref v3);Sip(ref v0,ref v1,ref v2,ref v3);Sip(ref v0,ref v1,ref v2,ref v3);var r4=v3;var r5=v2;var r6=v1;var r7=v0;var r8=Vector256.BitwiseOr(Vector256.BitwiseAnd(Vector256.Xor(r0,r4),V(~7UL)),V(3));for(int l=0;l<4;l++)Unsafe.WriteUnaligned((byte*)mem+l*16392+16384,r8[l]);var sp=V(16384);
 for(int rep=0;rep<4;rep++){
var bc=V(32);
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r7,r8),40);
r7=Sel(active,bf,r7);
r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r6,2),r4),r6);
r5=Sel(active,Vector256.Subtract(Ror(r5,54),r6),r5);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,2),r5),r3);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),r3),r2);
r0=Sel(active,Vector256.Multiply(Vector256.Xor(r0,V(9)),r7),r0);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(9)),r2),r1);
r4=Sel(active,Vector256.Add(Ror(r4,60),r1),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r4,r8),20);
r4=Sel(active,bf,r4);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r5,2),r3),r5);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,3),r5),r3);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(9)),r2),r7);
r0=Sel(active,Vector256.Multiply(Vector256.Xor(r0,V(1)),r7),r0);
r2=Sel(active,Vector256.Xor(Ror(r2,45),r4),r2);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),r6),r1);
r6=Sel(active,Vector256.Subtract(Ror(r6,51),r1),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r0,r8),10);
r0=Sel(active,bf,r0);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r2.AsInt64(),1).AsUInt64(),r7),r2);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),r4),r6);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r5,1),r3),r5);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(33)),r0),r1);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(9)),r6),r4);
r3=Sel(active,Vector256.Xor(Ror(r3,24),r1),r3);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),r5),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r5,r8),24);
r5=Sel(active,bf,r5);
r1=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r1.AsInt64(),3).AsUInt64(),r0),r1);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r6,1),r7),r6);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,3),r4),r3);
r2=Sel(active,Vector256.Xor(Ror(r2,37),r5),r2);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(33)),r3),r4);
r7=Sel(active,Vector256.Multiply(Vector256.Add(r7,V(9)),r2),r7);
r0=Sel(active,Vector256.Multiply(Vector256.Add(r0,V(1)),r1),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r1,r8),49);
r1=Sel(active,bf,r1);
r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r0,2),r1),r0);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r4,1),r6),r4);
r5=Sel(active,Vector256.Add(Ror(r5,10),r7),r5);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),r5),r7);
r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r6,V(9)),r3),r6);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),r0),r3);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),r4),r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r3,r8),63);
r3=Sel(active,bf,r3);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(1)),r0),r5);
r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r1.AsInt64(),3).AsUInt64(),r4),r1);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,2),r1),r4);
r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r2,2),r3),r2);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(9)),r6),r7);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),r7),r6);
r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r0,2),r2),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r4,r8),59);
r4=Sel(active,bf,r4);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),r7),r3);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r6,2),r3),r6);
r0=Sel(active,Vector256.Multiply(Vector256.Add(r0,V(9)),r6),r0);
r2=Sel(active,Vector256.Xor(Ror(r2,11),r0),r2);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),1).AsUInt64(),r2),r5);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(9)),r1),r7);
r1=Sel(active,Vector256.Subtract(Ror(r1,5),r5),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r0,r8),57);
r0=Sel(active,bf,r0);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r4.AsInt64(),3).AsUInt64(),r7),r4);
r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r2,2),r4),r2);
r6=Sel(active,Vector256.Multiply(Vector256.Add(r6,V(1)),r1),r6);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,2),r5),r3);
r5=Sel(active,Vector256.Xor(Ror(r5,6),r6),r5);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(1)),r3),r7);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),r0),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r6,r8),35);
r6=Sel(active,bf,r6);
r2=Sel(active,Vector256.Add(Ror(r2,18),r1),r2);
r7=Sel(active,Vector256.Multiply(Vector256.Add(r7,V(1)),r6),r7);
r3=Sel(active,Vector256.Multiply(Vector256.Add(r3,V(33)),r5),r3);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r0.AsInt64(),3).AsUInt64(),r4),r0);
r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r1,3),r3),r1);
r5=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r5,V(9)),r0),r5);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r4.AsInt64(),3).AsUInt64(),r2),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r4,r8),34);
r4=Sel(active,bf,r4);
r1=Sel(active,Vector256.Add(Ror(r1,16),r3),r1);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),1).AsUInt64(),r7),r6);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(33)),r1),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),r0),r2);
r0=Sel(active,Vector256.Add(Ror(r0,32),r2),r0);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r3.AsInt64(),1).AsUInt64(),r6),r3);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(9)),r4),r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r1,r8),1);
r1=Sel(active,bf,r1);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r7,1),r6),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(1)),r3),r2);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),r7),r3);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(33)),r4),r5);
r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r4,1),r5),r4);
r0=Sel(active,Vector256.Xor(Ror(r0,9),r1),r0);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),2).AsUInt64(),r2),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r3,r8),17);
r3=Sel(active,bf,r3);
r0=Sel(active,Vector256.Add(Ror(r0,35),r3),r0);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r5,1),r1),r5);
r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r4,V(33)),r7),r4);
r1=Sel(active,Vector256.Subtract(Ror(r1,7),r4),r1);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),r6),r2);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(33)),r5),r7);
r6=Sel(active,Vector256.Xor(Ror(r6,55),r0),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r0,r8),21);
r0=Sel(active,bf,r0);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r5.AsInt64(),3).AsUInt64(),r6),r5);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r2,2),r5),r2);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(9)),r2),r7);
r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r3.AsInt64(),1).AsUInt64(),r7),r3);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),r3),r1);
r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r6,V(1)),r0),r6);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r4.AsInt64(),1).AsUInt64(),r1),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r5,r8),41);
r5=Sel(active,bf,r5);
r6=Sel(active,Vector256.Add(Ror(r6,55),r3),r6);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r7,1),r6),r7);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(9)),r2),r4);
r1=Sel(active,Vector256.Subtract(Ror(r1,36),r0),r1);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r2,2),r4),r2);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),r5),r3);
r0=Sel(active,Vector256.Multiply(Vector256.Xor(r0,V(33)),r1),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r6,r8),19);
r6=Sel(active,bf,r6);
r3=Sel(active,Vector256.Add(Ror(r3,61),r6),r3);
r5=Sel(active,Vector256.Multiply(Vector256.Add(r5,V(1)),r2),r5);
r7=Sel(active,Vector256.Subtract(Ror(r7,54),r4),r7);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r2,2),r5),r2);
r4=Sel(active,Vector256.Multiply(Vector256.Add(r4,V(33)),r7),r4);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),r0),r1);
r0=Sel(active,Vector256.Add(Ror(r0,62),r3),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r1,r8),50);
r1=Sel(active,bf,r1);
r4=Sel(active,Vector256.Add(Ror(r4,35),r6),r4);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(1)),r5),r2);
r3=Sel(active,Vector256.Multiply(Vector256.Xor(r3,V(1)),r1),r3);
r6=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r6.AsInt64(),1).AsUInt64(),r3),r6);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),2).AsUInt64(),r2),r5);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(33)),r7),r0);
r7=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),r0),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r0,r8),31);
r0=Sel(active,bf,r0);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(33)),r6),r5);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),1).AsUInt64(),r1),r7);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(9)),r0),r3);
r6=Sel(active,Vector256.Add(Ror(r6,36),r2),r6);
r4=Sel(active,Vector256.Xor(Ror(r4,37),r5),r4);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),r3),r1);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r2,2),r7),r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r3,r8),58);
r3=Sel(active,bf,r3);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r7,3),r6),r7);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),r7),r6);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r5,1),r2),r5);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r4,1),r0),r4);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(1)),r5),r0);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r2.AsInt64(),1).AsUInt64(),r4),r2);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(33)),r3),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r5,r8),34);
r5=Sel(active,bf,r5);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),r6),r7);
r4=Sel(active,Vector256.Subtract(Ror(r4,1),r7),r4);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,2),r4),r0);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),r0),r2);
r1=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r1,2),r5),r1);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(33)),r2),r3);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),r3),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r2,r8),11);
r2=Sel(active,bf,r2);
r6=Sel(active,Vector256.Xor(Ror(r6,55),r7),r6);
r0=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r0.AsInt64(),3).AsUInt64(),r3),r0);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r7,1),r0),r7);
r4=Sel(active,Vector256.Multiply(Vector256.Add(r4,V(1)),r2),r4);
r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r3.AsInt64(),3).AsUInt64(),r4),r3);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(9)),r5),r1);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(33)),r1),r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r0,r8),24);
r0=Sel(active,bf,r0);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r2,1),r4),r2);
r6=Sel(active,Vector256.Multiply(Vector256.Add(r6,V(1)),r3),r6);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),r7),r1);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,1),r2),r4);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(9)),r1),r3);
r5=Sel(active,Vector256.Xor(Ror(r5,42),r6),r5);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r7.AsInt64(),3).AsUInt64(),r0),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r1,r8),20);
r1=Sel(active,bf,r1);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(9)),r2),r7);
r5=Sel(active,Vector256.Subtract(Ror(r5,44),r6),r5);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r2.AsInt64(),3).AsUInt64(),r5),r2);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r4.AsInt64(),2).AsUInt64(),r1),r4);
r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r0.AsInt64(),3).AsUInt64(),r3),r0);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(33)),r0),r3);
r6=Sel(active,Vector256.Multiply(Vector256.Add(r6,V(9)),r4),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r7,r8),22);
r7=Sel(active,bf,r7);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r5.AsInt64(),2).AsUInt64(),r0),r5);
r6=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r6,2),r4),r6);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),r6),r2);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(9)),r1),r0);
r3=Sel(active,Vector256.Xor(Ror(r3,26),r7),r3);
r4=Sel(active,Vector256.Add(Ror(r4,1),r3),r4);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(33)),r5),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r7,r8),59);
r7=Sel(active,bf,r7);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r0.AsInt64(),2).AsUInt64(),r1),r0);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r5,1),r2),r5);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),r0),r1);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),r3),r2);
r3=Sel(active,Vector256.Multiply(Vector256.Add(r3,V(33)),r5),r3);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),2).AsUInt64(),r7),r6);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,2),r6),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r7,r8),21);
r7=Sel(active,bf,r7);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),3).AsUInt64(),r3),r6);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(33)),r6),r1);
r5=Sel(active,Vector256.Multiply(Vector256.Add(r5,V(9)),r1),r5);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,3),r7),r0);
r4=Sel(active,Vector256.Subtract(Ror(r4,31),r5),r4);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),r4),r2);
r3=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r3,2),r2),r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r4,r8),33);
r4=Sel(active,bf,r4);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,2),r7),r3);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,3),r3),r0);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),r5),r1);
r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r6.AsInt64(),3).AsUInt64(),r4),r6);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(1)),r2),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),r1),r2);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r5,2),r6),r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r3,r8),4);
r3=Sel(active,bf,r3);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,2),r5),r4);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r5,3),r4),r5);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(1)),r3),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),r1),r2);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(9)),r2),r1);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,2),r6),r0);
r6=Sel(active,Vector256.Xor(Ror(r6,22),r0),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r4,r8),3);
r4=Sel(active,bf,r4);
r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r6,1),r1),r6);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),r4),r1);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(33)),r5),r0);
r3=Sel(active,Vector256.Subtract(Ror(r3,19),r0),r3);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r7,3),r2),r7);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),1).AsUInt64(),r6),r5);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),r7),r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r6,r8),3);
r6=Sel(active,bf,r6);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(9)),r3),r0);
r5=Sel(active,Vector256.Multiply(Vector256.Add(r5,V(33)),r7),r5);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(9)),r2),r3);
r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r2,3),r0),r2);
r1=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r1,3),r5),r1);
r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r4.AsInt64(),2).AsUInt64(),r6),r4);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),3).AsUInt64(),r4),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r0,r8),29);
r0=Sel(active,bf,r0);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(9)),r0),r1);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(9)),r5),r4);
r5=Sel(active,Vector256.Xor(Ror(r5,47),r4),r5);
r3=Sel(active,Vector256.Add(Ror(r3,13),r2),r3);
r6=Sel(active,Vector256.Subtract(Ror(r6,51),r3),r6);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),r7),r2);
r7=Sel(active,Vector256.Xor(Ror(r7,31),r1),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r6,r8),52);
r6=Sel(active,bf,r6);
r3=Sel(active,Vector256.Xor(Ror(r3,39),r7),r3);
r4=Sel(active,Vector256.Subtract(Ror(r4,56),r3),r4);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),3).AsUInt64(),r4),r5);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(1)),r5),r2);
r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r0,3),r6),r0);
r1=Sel(active,Vector256.Multiply(Vector256.Add(r1,V(33)),r2),r1);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(1)),r1),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
Store(mem,active,ref sp,r0,r1,r2,r3,r4,r5,r6,r7);
var bf=Ror(Vector256.Multiply(r7,r8),5);
r7=Sel(active,bf,r7);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r6,2),r1),r6);
r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r4,V(9)),r5),r4);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(1)),r0),r2);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r5.AsInt64(),3).AsUInt64(),r2),r5);
r0=Sel(active,Vector256.Xor(Ror(r0,62),r4),r0);
r1=Sel(active,Vector256.Multiply(Vector256.Add(r1,V(1)),r6),r1);
r3=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r3.AsInt64(),2).AsUInt64(),r7),r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
}
 for(int rep=0;rep<4;rep++){
var bc=V(32);
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r7,r8),40);
r7=Sel(active,bf,r7);
r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r6,2),Load(mem,r4)),r6);
r5=Sel(active,Vector256.Subtract(Ror(r5,54),Load(mem,r6)),r5);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,2),Load(mem,r5)),r3);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),Load(mem,r3)),r2);
r0=Sel(active,Vector256.Multiply(Vector256.Xor(r0,V(9)),Load(mem,r7)),r0);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(9)),Load(mem,r2)),r1);
r4=Sel(active,Vector256.Add(Ror(r4,60),Load(mem,r1)),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r4,r8),20);
r4=Sel(active,bf,r4);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r5,2),Load(mem,r3)),r5);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,3),Load(mem,r5)),r3);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(9)),Load(mem,r2)),r7);
r0=Sel(active,Vector256.Multiply(Vector256.Xor(r0,V(1)),Load(mem,r7)),r0);
r2=Sel(active,Vector256.Xor(Ror(r2,45),Load(mem,r4)),r2);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),Load(mem,r6)),r1);
r6=Sel(active,Vector256.Subtract(Ror(r6,51),Load(mem,r1)),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r0,r8),10);
r0=Sel(active,bf,r0);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r2.AsInt64(),1).AsUInt64(),Load(mem,r7)),r2);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),Load(mem,r4)),r6);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r5,1),Load(mem,r3)),r5);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(33)),Load(mem,r0)),r1);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(9)),Load(mem,r6)),r4);
r3=Sel(active,Vector256.Xor(Ror(r3,24),Load(mem,r1)),r3);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),Load(mem,r5)),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r5,r8),24);
r5=Sel(active,bf,r5);
r1=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r1.AsInt64(),3).AsUInt64(),Load(mem,r0)),r1);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r6,1),Load(mem,r7)),r6);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,3),Load(mem,r4)),r3);
r2=Sel(active,Vector256.Xor(Ror(r2,37),Load(mem,r5)),r2);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(33)),Load(mem,r3)),r4);
r7=Sel(active,Vector256.Multiply(Vector256.Add(r7,V(9)),Load(mem,r2)),r7);
r0=Sel(active,Vector256.Multiply(Vector256.Add(r0,V(1)),Load(mem,r1)),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r1,r8),49);
r1=Sel(active,bf,r1);
r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r0,2),Load(mem,r1)),r0);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r4,1),Load(mem,r6)),r4);
r5=Sel(active,Vector256.Add(Ror(r5,10),Load(mem,r7)),r5);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),Load(mem,r5)),r7);
r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r6,V(9)),Load(mem,r3)),r6);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),Load(mem,r0)),r3);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),Load(mem,r4)),r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r3,r8),63);
r3=Sel(active,bf,r3);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(1)),Load(mem,r0)),r5);
r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r1.AsInt64(),3).AsUInt64(),Load(mem,r4)),r1);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,2),Load(mem,r1)),r4);
r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r2,2),Load(mem,r3)),r2);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(9)),Load(mem,r6)),r7);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),Load(mem,r7)),r6);
r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r0,2),Load(mem,r2)),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r4,r8),59);
r4=Sel(active,bf,r4);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),Load(mem,r7)),r3);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r6,2),Load(mem,r3)),r6);
r0=Sel(active,Vector256.Multiply(Vector256.Add(r0,V(9)),Load(mem,r6)),r0);
r2=Sel(active,Vector256.Xor(Ror(r2,11),Load(mem,r0)),r2);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),1).AsUInt64(),Load(mem,r2)),r5);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(9)),Load(mem,r1)),r7);
r1=Sel(active,Vector256.Subtract(Ror(r1,5),Load(mem,r5)),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r0,r8),57);
r0=Sel(active,bf,r0);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r4.AsInt64(),3).AsUInt64(),Load(mem,r7)),r4);
r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r2,2),Load(mem,r4)),r2);
r6=Sel(active,Vector256.Multiply(Vector256.Add(r6,V(1)),Load(mem,r1)),r6);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,2),Load(mem,r5)),r3);
r5=Sel(active,Vector256.Xor(Ror(r5,6),Load(mem,r6)),r5);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(1)),Load(mem,r3)),r7);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),Load(mem,r0)),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r6,r8),35);
r6=Sel(active,bf,r6);
r2=Sel(active,Vector256.Add(Ror(r2,18),Load(mem,r1)),r2);
r7=Sel(active,Vector256.Multiply(Vector256.Add(r7,V(1)),Load(mem,r6)),r7);
r3=Sel(active,Vector256.Multiply(Vector256.Add(r3,V(33)),Load(mem,r5)),r3);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r0.AsInt64(),3).AsUInt64(),Load(mem,r4)),r0);
r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r1,3),Load(mem,r3)),r1);
r5=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r5,V(9)),Load(mem,r0)),r5);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r4.AsInt64(),3).AsUInt64(),Load(mem,r2)),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r4,r8),34);
r4=Sel(active,bf,r4);
r1=Sel(active,Vector256.Add(Ror(r1,16),Load(mem,r3)),r1);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),1).AsUInt64(),Load(mem,r7)),r6);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(33)),Load(mem,r1)),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),Load(mem,r0)),r2);
r0=Sel(active,Vector256.Add(Ror(r0,32),Load(mem,r2)),r0);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r3.AsInt64(),1).AsUInt64(),Load(mem,r6)),r3);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(9)),Load(mem,r4)),r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r1,r8),1);
r1=Sel(active,bf,r1);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r7,1),Load(mem,r6)),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(1)),Load(mem,r3)),r2);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),Load(mem,r7)),r3);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(33)),Load(mem,r4)),r5);
r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r4,1),Load(mem,r5)),r4);
r0=Sel(active,Vector256.Xor(Ror(r0,9),Load(mem,r1)),r0);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),2).AsUInt64(),Load(mem,r2)),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r3,r8),17);
r3=Sel(active,bf,r3);
r0=Sel(active,Vector256.Add(Ror(r0,35),Load(mem,r3)),r0);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r5,1),Load(mem,r1)),r5);
r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r4,V(33)),Load(mem,r7)),r4);
r1=Sel(active,Vector256.Subtract(Ror(r1,7),Load(mem,r4)),r1);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),Load(mem,r6)),r2);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(33)),Load(mem,r5)),r7);
r6=Sel(active,Vector256.Xor(Ror(r6,55),Load(mem,r0)),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r0,r8),21);
r0=Sel(active,bf,r0);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r5.AsInt64(),3).AsUInt64(),Load(mem,r6)),r5);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r2,2),Load(mem,r5)),r2);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(9)),Load(mem,r2)),r7);
r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r3.AsInt64(),1).AsUInt64(),Load(mem,r7)),r3);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),Load(mem,r3)),r1);
r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r6,V(1)),Load(mem,r0)),r6);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r4.AsInt64(),1).AsUInt64(),Load(mem,r1)),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r5,r8),41);
r5=Sel(active,bf,r5);
r6=Sel(active,Vector256.Add(Ror(r6,55),Load(mem,r3)),r6);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r7,1),Load(mem,r6)),r7);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(9)),Load(mem,r2)),r4);
r1=Sel(active,Vector256.Subtract(Ror(r1,36),Load(mem,r0)),r1);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r2,2),Load(mem,r4)),r2);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(1)),Load(mem,r5)),r3);
r0=Sel(active,Vector256.Multiply(Vector256.Xor(r0,V(33)),Load(mem,r1)),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r6,r8),19);
r6=Sel(active,bf,r6);
r3=Sel(active,Vector256.Add(Ror(r3,61),Load(mem,r6)),r3);
r5=Sel(active,Vector256.Multiply(Vector256.Add(r5,V(1)),Load(mem,r2)),r5);
r7=Sel(active,Vector256.Subtract(Ror(r7,54),Load(mem,r4)),r7);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r2,2),Load(mem,r5)),r2);
r4=Sel(active,Vector256.Multiply(Vector256.Add(r4,V(33)),Load(mem,r7)),r4);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),Load(mem,r0)),r1);
r0=Sel(active,Vector256.Add(Ror(r0,62),Load(mem,r3)),r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r1,r8),50);
r1=Sel(active,bf,r1);
r4=Sel(active,Vector256.Add(Ror(r4,35),Load(mem,r6)),r4);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(1)),Load(mem,r5)),r2);
r3=Sel(active,Vector256.Multiply(Vector256.Xor(r3,V(1)),Load(mem,r1)),r3);
r6=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r6.AsInt64(),1).AsUInt64(),Load(mem,r3)),r6);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),2).AsUInt64(),Load(mem,r2)),r5);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(33)),Load(mem,r7)),r0);
r7=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),Load(mem,r0)),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r0,r8),31);
r0=Sel(active,bf,r0);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(33)),Load(mem,r6)),r5);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),1).AsUInt64(),Load(mem,r1)),r7);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(9)),Load(mem,r0)),r3);
r6=Sel(active,Vector256.Add(Ror(r6,36),Load(mem,r2)),r6);
r4=Sel(active,Vector256.Xor(Ror(r4,37),Load(mem,r5)),r4);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),Load(mem,r3)),r1);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r2,2),Load(mem,r7)),r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r3,r8),58);
r3=Sel(active,bf,r3);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r7,3),Load(mem,r6)),r7);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),Load(mem,r7)),r6);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r5,1),Load(mem,r2)),r5);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r4,1),Load(mem,r0)),r4);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(1)),Load(mem,r5)),r0);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r2.AsInt64(),1).AsUInt64(),Load(mem,r4)),r2);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(33)),Load(mem,r3)),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r5,r8),34);
r5=Sel(active,bf,r5);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),2).AsUInt64(),Load(mem,r6)),r7);
r4=Sel(active,Vector256.Subtract(Ror(r4,1),Load(mem,r7)),r4);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,2),Load(mem,r4)),r0);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),Load(mem,r0)),r2);
r1=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r1,2),Load(mem,r5)),r1);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(33)),Load(mem,r2)),r3);
r6=Sel(active,Vector256.Multiply(Vector256.Xor(r6,V(33)),Load(mem,r3)),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r2,r8),11);
r2=Sel(active,bf,r2);
r6=Sel(active,Vector256.Xor(Ror(r6,55),Load(mem,r7)),r6);
r0=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r0.AsInt64(),3).AsUInt64(),Load(mem,r3)),r0);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r7,1),Load(mem,r0)),r7);
r4=Sel(active,Vector256.Multiply(Vector256.Add(r4,V(1)),Load(mem,r2)),r4);
r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r3.AsInt64(),3).AsUInt64(),Load(mem,r4)),r3);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(9)),Load(mem,r5)),r1);
r5=Sel(active,Vector256.Multiply(Vector256.Xor(r5,V(33)),Load(mem,r1)),r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r0,r8),24);
r0=Sel(active,bf,r0);
r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r2,1),Load(mem,r4)),r2);
r6=Sel(active,Vector256.Multiply(Vector256.Add(r6,V(1)),Load(mem,r3)),r6);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),Load(mem,r7)),r1);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,1),Load(mem,r2)),r4);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(9)),Load(mem,r1)),r3);
r5=Sel(active,Vector256.Xor(Ror(r5,42),Load(mem,r6)),r5);
r7=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r7.AsInt64(),3).AsUInt64(),Load(mem,r0)),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r1,r8),20);
r1=Sel(active,bf,r1);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(9)),Load(mem,r2)),r7);
r5=Sel(active,Vector256.Subtract(Ror(r5,44),Load(mem,r6)),r5);
r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r2.AsInt64(),3).AsUInt64(),Load(mem,r5)),r2);
r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r4.AsInt64(),2).AsUInt64(),Load(mem,r1)),r4);
r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r0.AsInt64(),3).AsUInt64(),Load(mem,r3)),r0);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(33)),Load(mem,r0)),r3);
r6=Sel(active,Vector256.Multiply(Vector256.Add(r6,V(9)),Load(mem,r4)),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r7,r8),22);
r7=Sel(active,bf,r7);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r5.AsInt64(),2).AsUInt64(),Load(mem,r0)),r5);
r6=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r6,2),Load(mem,r4)),r6);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),Load(mem,r6)),r2);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(9)),Load(mem,r1)),r0);
r3=Sel(active,Vector256.Xor(Ror(r3,26),Load(mem,r7)),r3);
r4=Sel(active,Vector256.Add(Ror(r4,1),Load(mem,r3)),r4);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(33)),Load(mem,r5)),r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r7,r8),59);
r7=Sel(active,bf,r7);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(r0.AsInt64(),2).AsUInt64(),Load(mem,r1)),r0);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r5,1),Load(mem,r2)),r5);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),Load(mem,r0)),r1);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),Load(mem,r3)),r2);
r3=Sel(active,Vector256.Multiply(Vector256.Add(r3,V(33)),Load(mem,r5)),r3);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),2).AsUInt64(),Load(mem,r7)),r6);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,2),Load(mem,r6)),r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r7,r8),21);
r7=Sel(active,bf,r7);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r6.AsInt64(),3).AsUInt64(),Load(mem,r3)),r6);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(33)),Load(mem,r6)),r1);
r5=Sel(active,Vector256.Multiply(Vector256.Add(r5,V(9)),Load(mem,r1)),r5);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,3),Load(mem,r7)),r0);
r4=Sel(active,Vector256.Subtract(Ror(r4,31),Load(mem,r5)),r4);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),Load(mem,r4)),r2);
r3=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r3,2),Load(mem,r2)),r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r4,r8),33);
r4=Sel(active,bf,r4);
r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r3,2),Load(mem,r7)),r3);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,3),Load(mem,r3)),r0);
r1=Sel(active,Vector256.Multiply(Vector256.Xor(r1,V(1)),Load(mem,r5)),r1);
r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r6.AsInt64(),3).AsUInt64(),Load(mem,r4)),r6);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(1)),Load(mem,r2)),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),Load(mem,r1)),r2);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r5,2),Load(mem,r6)),r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r3,r8),4);
r3=Sel(active,bf,r3);
r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r4,2),Load(mem,r5)),r4);
r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r5,3),Load(mem,r4)),r5);
r7=Sel(active,Vector256.Multiply(Vector256.Xor(r7,V(1)),Load(mem,r3)),r7);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(9)),Load(mem,r1)),r2);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(9)),Load(mem,r2)),r1);
r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r0,2),Load(mem,r6)),r0);
r6=Sel(active,Vector256.Xor(Ror(r6,22),Load(mem,r0)),r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r4,r8),3);
r4=Sel(active,bf,r4);
r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r6,1),Load(mem,r1)),r6);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(1)),Load(mem,r4)),r1);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(33)),Load(mem,r5)),r0);
r3=Sel(active,Vector256.Subtract(Ror(r3,19),Load(mem,r0)),r3);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r7,3),Load(mem,r2)),r7);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),1).AsUInt64(),Load(mem,r6)),r5);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(9)),Load(mem,r7)),r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r6,r8),3);
r6=Sel(active,bf,r6);
r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r0,V(9)),Load(mem,r3)),r0);
r5=Sel(active,Vector256.Multiply(Vector256.Add(r5,V(33)),Load(mem,r7)),r5);
r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r3,V(9)),Load(mem,r2)),r3);
r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r2,3),Load(mem,r0)),r2);
r1=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(r1,3),Load(mem,r5)),r1);
r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r4.AsInt64(),2).AsUInt64(),Load(mem,r6)),r4);
r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r7.AsInt64(),3).AsUInt64(),Load(mem,r4)),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r0,r8),29);
r0=Sel(active,bf,r0);
r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r1,V(9)),Load(mem,r0)),r1);
r4=Sel(active,Vector256.Multiply(Vector256.Xor(r4,V(9)),Load(mem,r5)),r4);
r5=Sel(active,Vector256.Xor(Ror(r5,47),Load(mem,r4)),r5);
r3=Sel(active,Vector256.Add(Ror(r3,13),Load(mem,r2)),r3);
r6=Sel(active,Vector256.Subtract(Ror(r6,51),Load(mem,r3)),r6);
r2=Sel(active,Vector256.Multiply(Vector256.Xor(r2,V(33)),Load(mem,r7)),r2);
r7=Sel(active,Vector256.Xor(Ror(r7,31),Load(mem,r1)),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r6,r8),52);
r6=Sel(active,bf,r6);
r3=Sel(active,Vector256.Xor(Ror(r3,39),Load(mem,r7)),r3);
r4=Sel(active,Vector256.Subtract(Ror(r4,56),Load(mem,r3)),r4);
r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r5.AsInt64(),3).AsUInt64(),Load(mem,r4)),r5);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(1)),Load(mem,r5)),r2);
r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(r0,3),Load(mem,r6)),r0);
r1=Sel(active,Vector256.Multiply(Vector256.Add(r1,V(33)),Load(mem,r2)),r1);
r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r7,V(1)),Load(mem,r1)),r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
{
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(r7,r8),5);
r7=Sel(active,bf,r7);
r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(r6,2),Load(mem,r1)),r6);
r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r4,V(9)),Load(mem,r5)),r4);
r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(r2,V(1)),Load(mem,r0)),r2);
r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(r5.AsInt64(),3).AsUInt64(),Load(mem,r2)),r5);
r0=Sel(active,Vector256.Xor(Ror(r0,62),Load(mem,r4)),r0);
r1=Sel(active,Vector256.Multiply(Vector256.Add(r1,V(1)),Load(mem,r6)),r1);
r3=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(r3.AsInt64(),2).AsUInt64(),Load(mem,r7)),r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
}
 v0=r0;v1=r1;v2=r2;v3=r3;Sip(ref v0,ref v1,ref v2,ref v3);var out3=v3;v0=r4;v1=r5;v2=r6;v3=r7;Sip(ref v0,ref v1,ref v2,ref v3);return Vector256.Xor(Vector256.Xor(out3,v3),r8);
 }
}
class Program{static unsafe void Main(){Console.WriteLine($"avx2={Avx2.IsSupported} avx512dq={Avx512DQ.IsSupported}");nint mem=(nint)NativeMemory.Alloc((nuint)(4*16392));var z=HW.Hash4(123456,mem);Console.WriteLine($"CHECK {z[0]:x16} {z[1]:x16} {z[2]:x16} {z[3]:x16}");for(int i=0;i<1000;i++)_=HW.Hash4((ulong)(i*4),mem);const int H=200000;var sw=Stopwatch.StartNew();ulong sink=0;for(int i=0;i<H/4;i++){var q=HW.Hash4((ulong)(i*4),mem);sink^=q[0]^q[1]^q[2]^q[3];}sw.Stop();Console.WriteLine($"RATE {H/sw.Elapsed.TotalSeconds:F1} sink={sink:x16}");NativeMemory.Free((void*)mem);}}
