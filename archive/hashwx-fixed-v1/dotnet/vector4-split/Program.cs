using System;using System.Diagnostics;using System.Numerics;using System.Runtime.CompilerServices;using System.Runtime.InteropServices;using System.Runtime.Intrinsics;using System.Runtime.Intrinsics.X86;
struct S{public Vector256<ulong> r0,r1,r2,r3,r4,r5,r6,r7,r8;}
static class HW{const ulong K0=1663540288323457296UL,K1=2242261671028070680UL;static readonly Vector256<ulong> ZERO=Vector256<ulong>.Zero,ONE=Vector256.Create(1UL),ALL=Vector256.Create(ulong.MaxValue),BASE=Vector256.Create(0UL,16392UL,32784UL,49176UL),MASK=Vector256.Create(16383UL);
[MethodImpl(MethodImplOptions.AggressiveInlining)]static Vector256<ulong> V(ulong x)=>Vector256.Create(x);[MethodImpl(MethodImplOptions.AggressiveInlining)]static Vector256<ulong> Sel(Vector256<ulong>m,Vector256<ulong>a,Vector256<ulong>b)=>Vector256.ConditionalSelect(m,a,b);[MethodImpl(MethodImplOptions.AggressiveInlining)]static Vector256<ulong> Ror(Vector256<ulong>x,int n)=>Vector256.BitwiseOr(Vector256.ShiftRightLogical(x,n),Vector256.ShiftLeft(x,64-n));[MethodImpl(MethodImplOptions.AggressiveInlining)]static Vector256<ulong> NotZero(Vector256<ulong>x)=>Vector256.Xor(Vector256.Equals(x,ZERO),ALL);[MethodImpl(MethodImplOptions.AggressiveInlining)]static bool None(Vector256<ulong>x)=>(x[0]|x[1]|x[2]|x[3])==0;
[MethodImpl(MethodImplOptions.AggressiveInlining)]static unsafe Vector256<ulong> Load(nint mem,Vector256<ulong>a)=>Avx2.GatherVector256((ulong*)mem,Vector256.Add(BASE,Vector256.BitwiseAnd(a,MASK)).AsInt64(),1);
[MethodImpl(MethodImplOptions.AggressiveInlining)]static unsafe void Store(nint mem,Vector256<ulong>active,ref Vector256<ulong>sp,S s){var n=Vector256.Subtract(sp,V(64));sp=Sel(active,n,sp);for(int l=0;l<4;l++){if(active[l]==0)continue;byte*p=(byte*)mem+l*16392+(nint)sp[l];Unsafe.WriteUnaligned(p+56,s.r0[l]);Unsafe.WriteUnaligned(p+48,s.r1[l]);Unsafe.WriteUnaligned(p+40,s.r2[l]);Unsafe.WriteUnaligned(p+32,s.r3[l]);Unsafe.WriteUnaligned(p+24,s.r4[l]);Unsafe.WriteUnaligned(p+16,s.r5[l]);Unsafe.WriteUnaligned(p+8,s.r6[l]);Unsafe.WriteUnaligned(p,s.r7[l]);}}
[MethodImpl(MethodImplOptions.AggressiveInlining)]static void Sip(ref Vector256<ulong>a,ref Vector256<ulong>b,ref Vector256<ulong>c,ref Vector256<ulong>d){a=Vector256.Add(a,b);c=Vector256.Add(c,d);b=Ror(b,51);d=Ror(d,48);b=Vector256.Xor(b,a);d=Vector256.Xor(d,c);a=Ror(a,32);c=Vector256.Add(c,b);a=Vector256.Add(a,d);b=Ror(b,47);d=Ror(d,43);b=Vector256.Xor(b,c);d=Vector256.Xor(d,a);c=Ror(c,32);}
[MethodImpl(MethodImplOptions.NoInlining)] static void P0W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r7,s.r8),40);
s.r7=Sel(active,bf,s.r7);
s.r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r6,2),s.r4),s.r6);
s.r5=Sel(active,Vector256.Subtract(Ror(s.r5,54),s.r6),s.r5);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,2),s.r5),s.r3);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),s.r3),s.r2);
s.r0=Sel(active,Vector256.Multiply(Vector256.Xor(s.r0,V(9)),s.r7),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(9)),s.r2),s.r1);
s.r4=Sel(active,Vector256.Add(Ror(s.r4,60),s.r1),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P1W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r4,s.r8),20);
s.r4=Sel(active,bf,s.r4);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r5,2),s.r3),s.r5);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,3),s.r5),s.r3);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(9)),s.r2),s.r7);
s.r0=Sel(active,Vector256.Multiply(Vector256.Xor(s.r0,V(1)),s.r7),s.r0);
s.r2=Sel(active,Vector256.Xor(Ror(s.r2,45),s.r4),s.r2);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),s.r6),s.r1);
s.r6=Sel(active,Vector256.Subtract(Ror(s.r6,51),s.r1),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P2W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r0,s.r8),10);
s.r0=Sel(active,bf,s.r0);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r2.AsInt64(),1).AsUInt64(),s.r7),s.r2);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),s.r4),s.r6);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r5,1),s.r3),s.r5);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(33)),s.r0),s.r1);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(9)),s.r6),s.r4);
s.r3=Sel(active,Vector256.Xor(Ror(s.r3,24),s.r1),s.r3);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),s.r5),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P3W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r5,s.r8),24);
s.r5=Sel(active,bf,s.r5);
s.r1=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r1.AsInt64(),3).AsUInt64(),s.r0),s.r1);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r6,1),s.r7),s.r6);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,3),s.r4),s.r3);
s.r2=Sel(active,Vector256.Xor(Ror(s.r2,37),s.r5),s.r2);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(33)),s.r3),s.r4);
s.r7=Sel(active,Vector256.Multiply(Vector256.Add(s.r7,V(9)),s.r2),s.r7);
s.r0=Sel(active,Vector256.Multiply(Vector256.Add(s.r0,V(1)),s.r1),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P4W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r1,s.r8),49);
s.r1=Sel(active,bf,s.r1);
s.r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r0,2),s.r1),s.r0);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r4,1),s.r6),s.r4);
s.r5=Sel(active,Vector256.Add(Ror(s.r5,10),s.r7),s.r5);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),s.r5),s.r7);
s.r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r6,V(9)),s.r3),s.r6);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),s.r0),s.r3);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),s.r4),s.r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P5W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r3,s.r8),63);
s.r3=Sel(active,bf,s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(1)),s.r0),s.r5);
s.r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r1.AsInt64(),3).AsUInt64(),s.r4),s.r1);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,2),s.r1),s.r4);
s.r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r2,2),s.r3),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(9)),s.r6),s.r7);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),s.r7),s.r6);
s.r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r0,2),s.r2),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P6W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r4,s.r8),59);
s.r4=Sel(active,bf,s.r4);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),s.r7),s.r3);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r6,2),s.r3),s.r6);
s.r0=Sel(active,Vector256.Multiply(Vector256.Add(s.r0,V(9)),s.r6),s.r0);
s.r2=Sel(active,Vector256.Xor(Ror(s.r2,11),s.r0),s.r2);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),1).AsUInt64(),s.r2),s.r5);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(9)),s.r1),s.r7);
s.r1=Sel(active,Vector256.Subtract(Ror(s.r1,5),s.r5),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P7W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r0,s.r8),57);
s.r0=Sel(active,bf,s.r0);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),3).AsUInt64(),s.r7),s.r4);
s.r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r2,2),s.r4),s.r2);
s.r6=Sel(active,Vector256.Multiply(Vector256.Add(s.r6,V(1)),s.r1),s.r6);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,2),s.r5),s.r3);
s.r5=Sel(active,Vector256.Xor(Ror(s.r5,6),s.r6),s.r5);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(1)),s.r3),s.r7);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),s.r0),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P8W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r6,s.r8),35);
s.r6=Sel(active,bf,s.r6);
s.r2=Sel(active,Vector256.Add(Ror(s.r2,18),s.r1),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.Add(s.r7,V(1)),s.r6),s.r7);
s.r3=Sel(active,Vector256.Multiply(Vector256.Add(s.r3,V(33)),s.r5),s.r3);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),3).AsUInt64(),s.r4),s.r0);
s.r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r1,3),s.r3),s.r1);
s.r5=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r5,V(9)),s.r0),s.r5);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),3).AsUInt64(),s.r2),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P9W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r4,s.r8),34);
s.r4=Sel(active,bf,s.r4);
s.r1=Sel(active,Vector256.Add(Ror(s.r1,16),s.r3),s.r1);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),1).AsUInt64(),s.r7),s.r6);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(33)),s.r1),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),s.r0),s.r2);
s.r0=Sel(active,Vector256.Add(Ror(s.r0,32),s.r2),s.r0);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),1).AsUInt64(),s.r6),s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(9)),s.r4),s.r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P10W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r1,s.r8),1);
s.r1=Sel(active,bf,s.r1);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r7,1),s.r6),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(1)),s.r3),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),s.r7),s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(33)),s.r4),s.r5);
s.r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r4,1),s.r5),s.r4);
s.r0=Sel(active,Vector256.Xor(Ror(s.r0,9),s.r1),s.r0);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),2).AsUInt64(),s.r2),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P11W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r3,s.r8),17);
s.r3=Sel(active,bf,s.r3);
s.r0=Sel(active,Vector256.Add(Ror(s.r0,35),s.r3),s.r0);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r5,1),s.r1),s.r5);
s.r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r4,V(33)),s.r7),s.r4);
s.r1=Sel(active,Vector256.Subtract(Ror(s.r1,7),s.r4),s.r1);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),s.r6),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(33)),s.r5),s.r7);
s.r6=Sel(active,Vector256.Xor(Ror(s.r6,55),s.r0),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P12W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r0,s.r8),21);
s.r0=Sel(active,bf,s.r0);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),3).AsUInt64(),s.r6),s.r5);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r2,2),s.r5),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(9)),s.r2),s.r7);
s.r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),1).AsUInt64(),s.r7),s.r3);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),s.r3),s.r1);
s.r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r6,V(1)),s.r0),s.r6);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),1).AsUInt64(),s.r1),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P13W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r5,s.r8),41);
s.r5=Sel(active,bf,s.r5);
s.r6=Sel(active,Vector256.Add(Ror(s.r6,55),s.r3),s.r6);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r7,1),s.r6),s.r7);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(9)),s.r2),s.r4);
s.r1=Sel(active,Vector256.Subtract(Ror(s.r1,36),s.r0),s.r1);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r2,2),s.r4),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),s.r5),s.r3);
s.r0=Sel(active,Vector256.Multiply(Vector256.Xor(s.r0,V(33)),s.r1),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P14W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r6,s.r8),19);
s.r6=Sel(active,bf,s.r6);
s.r3=Sel(active,Vector256.Add(Ror(s.r3,61),s.r6),s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Add(s.r5,V(1)),s.r2),s.r5);
s.r7=Sel(active,Vector256.Subtract(Ror(s.r7,54),s.r4),s.r7);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r2,2),s.r5),s.r2);
s.r4=Sel(active,Vector256.Multiply(Vector256.Add(s.r4,V(33)),s.r7),s.r4);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),s.r0),s.r1);
s.r0=Sel(active,Vector256.Add(Ror(s.r0,62),s.r3),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P15W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r1,s.r8),50);
s.r1=Sel(active,bf,s.r1);
s.r4=Sel(active,Vector256.Add(Ror(s.r4,35),s.r6),s.r4);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(1)),s.r5),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.Xor(s.r3,V(1)),s.r1),s.r3);
s.r6=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),1).AsUInt64(),s.r3),s.r6);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),2).AsUInt64(),s.r2),s.r5);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(33)),s.r7),s.r0);
s.r7=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),s.r0),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P16W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r0,s.r8),31);
s.r0=Sel(active,bf,s.r0);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(33)),s.r6),s.r5);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),1).AsUInt64(),s.r1),s.r7);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(9)),s.r0),s.r3);
s.r6=Sel(active,Vector256.Add(Ror(s.r6,36),s.r2),s.r6);
s.r4=Sel(active,Vector256.Xor(Ror(s.r4,37),s.r5),s.r4);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),s.r3),s.r1);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r2,2),s.r7),s.r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P17W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r3,s.r8),58);
s.r3=Sel(active,bf,s.r3);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r7,3),s.r6),s.r7);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),s.r7),s.r6);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r5,1),s.r2),s.r5);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r4,1),s.r0),s.r4);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(1)),s.r5),s.r0);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r2.AsInt64(),1).AsUInt64(),s.r4),s.r2);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(33)),s.r3),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P18W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r5,s.r8),34);
s.r5=Sel(active,bf,s.r5);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),s.r6),s.r7);
s.r4=Sel(active,Vector256.Subtract(Ror(s.r4,1),s.r7),s.r4);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,2),s.r4),s.r0);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),s.r0),s.r2);
s.r1=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r1,2),s.r5),s.r1);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(33)),s.r2),s.r3);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),s.r3),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P19W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r2,s.r8),11);
s.r2=Sel(active,bf,s.r2);
s.r6=Sel(active,Vector256.Xor(Ror(s.r6,55),s.r7),s.r6);
s.r0=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),3).AsUInt64(),s.r3),s.r0);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r7,1),s.r0),s.r7);
s.r4=Sel(active,Vector256.Multiply(Vector256.Add(s.r4,V(1)),s.r2),s.r4);
s.r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),3).AsUInt64(),s.r4),s.r3);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(9)),s.r5),s.r1);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(33)),s.r1),s.r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P20W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r0,s.r8),24);
s.r0=Sel(active,bf,s.r0);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r2,1),s.r4),s.r2);
s.r6=Sel(active,Vector256.Multiply(Vector256.Add(s.r6,V(1)),s.r3),s.r6);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),s.r7),s.r1);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,1),s.r2),s.r4);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(9)),s.r1),s.r3);
s.r5=Sel(active,Vector256.Xor(Ror(s.r5,42),s.r6),s.r5);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),3).AsUInt64(),s.r0),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P21W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r1,s.r8),20);
s.r1=Sel(active,bf,s.r1);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(9)),s.r2),s.r7);
s.r5=Sel(active,Vector256.Subtract(Ror(s.r5,44),s.r6),s.r5);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r2.AsInt64(),3).AsUInt64(),s.r5),s.r2);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),2).AsUInt64(),s.r1),s.r4);
s.r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),3).AsUInt64(),s.r3),s.r0);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(33)),s.r0),s.r3);
s.r6=Sel(active,Vector256.Multiply(Vector256.Add(s.r6,V(9)),s.r4),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P22W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r7,s.r8),22);
s.r7=Sel(active,bf,s.r7);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),2).AsUInt64(),s.r0),s.r5);
s.r6=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r6,2),s.r4),s.r6);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),s.r6),s.r2);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(9)),s.r1),s.r0);
s.r3=Sel(active,Vector256.Xor(Ror(s.r3,26),s.r7),s.r3);
s.r4=Sel(active,Vector256.Add(Ror(s.r4,1),s.r3),s.r4);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(33)),s.r5),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P23W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r7,s.r8),59);
s.r7=Sel(active,bf,s.r7);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),2).AsUInt64(),s.r1),s.r0);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r5,1),s.r2),s.r5);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),s.r0),s.r1);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),s.r3),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.Add(s.r3,V(33)),s.r5),s.r3);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),2).AsUInt64(),s.r7),s.r6);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,2),s.r6),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P24W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r7,s.r8),21);
s.r7=Sel(active,bf,s.r7);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),3).AsUInt64(),s.r3),s.r6);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(33)),s.r6),s.r1);
s.r5=Sel(active,Vector256.Multiply(Vector256.Add(s.r5,V(9)),s.r1),s.r5);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,3),s.r7),s.r0);
s.r4=Sel(active,Vector256.Subtract(Ror(s.r4,31),s.r5),s.r4);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),s.r4),s.r2);
s.r3=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r3,2),s.r2),s.r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P25W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r4,s.r8),33);
s.r4=Sel(active,bf,s.r4);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,2),s.r7),s.r3);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,3),s.r3),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),s.r5),s.r1);
s.r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),3).AsUInt64(),s.r4),s.r6);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(1)),s.r2),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),s.r1),s.r2);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r5,2),s.r6),s.r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P26W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r3,s.r8),4);
s.r3=Sel(active,bf,s.r3);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,2),s.r5),s.r4);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r5,3),s.r4),s.r5);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(1)),s.r3),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),s.r1),s.r2);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(9)),s.r2),s.r1);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,2),s.r6),s.r0);
s.r6=Sel(active,Vector256.Xor(Ror(s.r6,22),s.r0),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P27W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r4,s.r8),3);
s.r4=Sel(active,bf,s.r4);
s.r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r6,1),s.r1),s.r6);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),s.r4),s.r1);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(33)),s.r5),s.r0);
s.r3=Sel(active,Vector256.Subtract(Ror(s.r3,19),s.r0),s.r3);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r7,3),s.r2),s.r7);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),1).AsUInt64(),s.r6),s.r5);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),s.r7),s.r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P28W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r6,s.r8),3);
s.r6=Sel(active,bf,s.r6);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(9)),s.r3),s.r0);
s.r5=Sel(active,Vector256.Multiply(Vector256.Add(s.r5,V(33)),s.r7),s.r5);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(9)),s.r2),s.r3);
s.r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r2,3),s.r0),s.r2);
s.r1=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r1,3),s.r5),s.r1);
s.r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),2).AsUInt64(),s.r6),s.r4);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),3).AsUInt64(),s.r4),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P29W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r0,s.r8),29);
s.r0=Sel(active,bf,s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(9)),s.r0),s.r1);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(9)),s.r5),s.r4);
s.r5=Sel(active,Vector256.Xor(Ror(s.r5,47),s.r4),s.r5);
s.r3=Sel(active,Vector256.Add(Ror(s.r3,13),s.r2),s.r3);
s.r6=Sel(active,Vector256.Subtract(Ror(s.r6,51),s.r3),s.r6);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),s.r7),s.r2);
s.r7=Sel(active,Vector256.Xor(Ror(s.r7,31),s.r1),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P30W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r6,s.r8),52);
s.r6=Sel(active,bf,s.r6);
s.r3=Sel(active,Vector256.Xor(Ror(s.r3,39),s.r7),s.r3);
s.r4=Sel(active,Vector256.Subtract(Ror(s.r4,56),s.r3),s.r4);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),3).AsUInt64(),s.r4),s.r5);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(1)),s.r5),s.r2);
s.r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r0,3),s.r6),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.Add(s.r1,V(33)),s.r2),s.r1);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(1)),s.r1),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P31W(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
Store(mem,active,ref sp,s);
var bf=Ror(Vector256.Multiply(s.r7,s.r8),5);
s.r7=Sel(active,bf,s.r7);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r6,2),s.r1),s.r6);
s.r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r4,V(9)),s.r5),s.r4);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(1)),s.r0),s.r2);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),3).AsUInt64(),s.r2),s.r5);
s.r0=Sel(active,Vector256.Xor(Ror(s.r0,62),s.r4),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.Add(s.r1,V(1)),s.r6),s.r1);
s.r3=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),2).AsUInt64(),s.r7),s.r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P0R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r7,s.r8),40);
s.r7=Sel(active,bf,s.r7);
s.r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r6,2),Load(mem,s.r4)),s.r6);
s.r5=Sel(active,Vector256.Subtract(Ror(s.r5,54),Load(mem,s.r6)),s.r5);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,2),Load(mem,s.r5)),s.r3);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),Load(mem,s.r3)),s.r2);
s.r0=Sel(active,Vector256.Multiply(Vector256.Xor(s.r0,V(9)),Load(mem,s.r7)),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(9)),Load(mem,s.r2)),s.r1);
s.r4=Sel(active,Vector256.Add(Ror(s.r4,60),Load(mem,s.r1)),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P1R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r4,s.r8),20);
s.r4=Sel(active,bf,s.r4);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r5,2),Load(mem,s.r3)),s.r5);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,3),Load(mem,s.r5)),s.r3);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(9)),Load(mem,s.r2)),s.r7);
s.r0=Sel(active,Vector256.Multiply(Vector256.Xor(s.r0,V(1)),Load(mem,s.r7)),s.r0);
s.r2=Sel(active,Vector256.Xor(Ror(s.r2,45),Load(mem,s.r4)),s.r2);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),Load(mem,s.r6)),s.r1);
s.r6=Sel(active,Vector256.Subtract(Ror(s.r6,51),Load(mem,s.r1)),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P2R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r0,s.r8),10);
s.r0=Sel(active,bf,s.r0);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r2.AsInt64(),1).AsUInt64(),Load(mem,s.r7)),s.r2);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),Load(mem,s.r4)),s.r6);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r5,1),Load(mem,s.r3)),s.r5);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(33)),Load(mem,s.r0)),s.r1);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(9)),Load(mem,s.r6)),s.r4);
s.r3=Sel(active,Vector256.Xor(Ror(s.r3,24),Load(mem,s.r1)),s.r3);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),Load(mem,s.r5)),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P3R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r5,s.r8),24);
s.r5=Sel(active,bf,s.r5);
s.r1=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r1.AsInt64(),3).AsUInt64(),Load(mem,s.r0)),s.r1);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r6,1),Load(mem,s.r7)),s.r6);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,3),Load(mem,s.r4)),s.r3);
s.r2=Sel(active,Vector256.Xor(Ror(s.r2,37),Load(mem,s.r5)),s.r2);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(33)),Load(mem,s.r3)),s.r4);
s.r7=Sel(active,Vector256.Multiply(Vector256.Add(s.r7,V(9)),Load(mem,s.r2)),s.r7);
s.r0=Sel(active,Vector256.Multiply(Vector256.Add(s.r0,V(1)),Load(mem,s.r1)),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P4R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r1,s.r8),49);
s.r1=Sel(active,bf,s.r1);
s.r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r0,2),Load(mem,s.r1)),s.r0);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r4,1),Load(mem,s.r6)),s.r4);
s.r5=Sel(active,Vector256.Add(Ror(s.r5,10),Load(mem,s.r7)),s.r5);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),Load(mem,s.r5)),s.r7);
s.r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r6,V(9)),Load(mem,s.r3)),s.r6);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),Load(mem,s.r0)),s.r3);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),Load(mem,s.r4)),s.r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P5R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r3,s.r8),63);
s.r3=Sel(active,bf,s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(1)),Load(mem,s.r0)),s.r5);
s.r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r1.AsInt64(),3).AsUInt64(),Load(mem,s.r4)),s.r1);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,2),Load(mem,s.r1)),s.r4);
s.r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r2,2),Load(mem,s.r3)),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(9)),Load(mem,s.r6)),s.r7);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),Load(mem,s.r7)),s.r6);
s.r0=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r0,2),Load(mem,s.r2)),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P6R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r4,s.r8),59);
s.r4=Sel(active,bf,s.r4);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),Load(mem,s.r7)),s.r3);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r6,2),Load(mem,s.r3)),s.r6);
s.r0=Sel(active,Vector256.Multiply(Vector256.Add(s.r0,V(9)),Load(mem,s.r6)),s.r0);
s.r2=Sel(active,Vector256.Xor(Ror(s.r2,11),Load(mem,s.r0)),s.r2);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),1).AsUInt64(),Load(mem,s.r2)),s.r5);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(9)),Load(mem,s.r1)),s.r7);
s.r1=Sel(active,Vector256.Subtract(Ror(s.r1,5),Load(mem,s.r5)),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P7R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r0,s.r8),57);
s.r0=Sel(active,bf,s.r0);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),3).AsUInt64(),Load(mem,s.r7)),s.r4);
s.r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r2,2),Load(mem,s.r4)),s.r2);
s.r6=Sel(active,Vector256.Multiply(Vector256.Add(s.r6,V(1)),Load(mem,s.r1)),s.r6);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,2),Load(mem,s.r5)),s.r3);
s.r5=Sel(active,Vector256.Xor(Ror(s.r5,6),Load(mem,s.r6)),s.r5);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(1)),Load(mem,s.r3)),s.r7);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),Load(mem,s.r0)),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P8R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r6,s.r8),35);
s.r6=Sel(active,bf,s.r6);
s.r2=Sel(active,Vector256.Add(Ror(s.r2,18),Load(mem,s.r1)),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.Add(s.r7,V(1)),Load(mem,s.r6)),s.r7);
s.r3=Sel(active,Vector256.Multiply(Vector256.Add(s.r3,V(33)),Load(mem,s.r5)),s.r3);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),3).AsUInt64(),Load(mem,s.r4)),s.r0);
s.r1=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r1,3),Load(mem,s.r3)),s.r1);
s.r5=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r5,V(9)),Load(mem,s.r0)),s.r5);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),3).AsUInt64(),Load(mem,s.r2)),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P9R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r4,s.r8),34);
s.r4=Sel(active,bf,s.r4);
s.r1=Sel(active,Vector256.Add(Ror(s.r1,16),Load(mem,s.r3)),s.r1);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),1).AsUInt64(),Load(mem,s.r7)),s.r6);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(33)),Load(mem,s.r1)),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),Load(mem,s.r0)),s.r2);
s.r0=Sel(active,Vector256.Add(Ror(s.r0,32),Load(mem,s.r2)),s.r0);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),1).AsUInt64(),Load(mem,s.r6)),s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(9)),Load(mem,s.r4)),s.r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P10R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r1,s.r8),1);
s.r1=Sel(active,bf,s.r1);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r7,1),Load(mem,s.r6)),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(1)),Load(mem,s.r3)),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),Load(mem,s.r7)),s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(33)),Load(mem,s.r4)),s.r5);
s.r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r4,1),Load(mem,s.r5)),s.r4);
s.r0=Sel(active,Vector256.Xor(Ror(s.r0,9),Load(mem,s.r1)),s.r0);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),2).AsUInt64(),Load(mem,s.r2)),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P11R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r3,s.r8),17);
s.r3=Sel(active,bf,s.r3);
s.r0=Sel(active,Vector256.Add(Ror(s.r0,35),Load(mem,s.r3)),s.r0);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r5,1),Load(mem,s.r1)),s.r5);
s.r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r4,V(33)),Load(mem,s.r7)),s.r4);
s.r1=Sel(active,Vector256.Subtract(Ror(s.r1,7),Load(mem,s.r4)),s.r1);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),Load(mem,s.r6)),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(33)),Load(mem,s.r5)),s.r7);
s.r6=Sel(active,Vector256.Xor(Ror(s.r6,55),Load(mem,s.r0)),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P12R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r0,s.r8),21);
s.r0=Sel(active,bf,s.r0);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),3).AsUInt64(),Load(mem,s.r6)),s.r5);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r2,2),Load(mem,s.r5)),s.r2);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(9)),Load(mem,s.r2)),s.r7);
s.r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),1).AsUInt64(),Load(mem,s.r7)),s.r3);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),Load(mem,s.r3)),s.r1);
s.r6=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r6,V(1)),Load(mem,s.r0)),s.r6);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),1).AsUInt64(),Load(mem,s.r1)),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P13R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r5,s.r8),41);
s.r5=Sel(active,bf,s.r5);
s.r6=Sel(active,Vector256.Add(Ror(s.r6,55),Load(mem,s.r3)),s.r6);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r7,1),Load(mem,s.r6)),s.r7);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(9)),Load(mem,s.r2)),s.r4);
s.r1=Sel(active,Vector256.Subtract(Ror(s.r1,36),Load(mem,s.r0)),s.r1);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r2,2),Load(mem,s.r4)),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(1)),Load(mem,s.r5)),s.r3);
s.r0=Sel(active,Vector256.Multiply(Vector256.Xor(s.r0,V(33)),Load(mem,s.r1)),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P14R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r6,s.r8),19);
s.r6=Sel(active,bf,s.r6);
s.r3=Sel(active,Vector256.Add(Ror(s.r3,61),Load(mem,s.r6)),s.r3);
s.r5=Sel(active,Vector256.Multiply(Vector256.Add(s.r5,V(1)),Load(mem,s.r2)),s.r5);
s.r7=Sel(active,Vector256.Subtract(Ror(s.r7,54),Load(mem,s.r4)),s.r7);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r2,2),Load(mem,s.r5)),s.r2);
s.r4=Sel(active,Vector256.Multiply(Vector256.Add(s.r4,V(33)),Load(mem,s.r7)),s.r4);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),Load(mem,s.r0)),s.r1);
s.r0=Sel(active,Vector256.Add(Ror(s.r0,62),Load(mem,s.r3)),s.r0);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P15R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r1,s.r8),50);
s.r1=Sel(active,bf,s.r1);
s.r4=Sel(active,Vector256.Add(Ror(s.r4,35),Load(mem,s.r6)),s.r4);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(1)),Load(mem,s.r5)),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.Xor(s.r3,V(1)),Load(mem,s.r1)),s.r3);
s.r6=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),1).AsUInt64(),Load(mem,s.r3)),s.r6);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),2).AsUInt64(),Load(mem,s.r2)),s.r5);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(33)),Load(mem,s.r7)),s.r0);
s.r7=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),Load(mem,s.r0)),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P16R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r0,s.r8),31);
s.r0=Sel(active,bf,s.r0);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(33)),Load(mem,s.r6)),s.r5);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),1).AsUInt64(),Load(mem,s.r1)),s.r7);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(9)),Load(mem,s.r0)),s.r3);
s.r6=Sel(active,Vector256.Add(Ror(s.r6,36),Load(mem,s.r2)),s.r6);
s.r4=Sel(active,Vector256.Xor(Ror(s.r4,37),Load(mem,s.r5)),s.r4);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),Load(mem,s.r3)),s.r1);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r2,2),Load(mem,s.r7)),s.r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P17R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r3,s.r8),58);
s.r3=Sel(active,bf,s.r3);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r7,3),Load(mem,s.r6)),s.r7);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),Load(mem,s.r7)),s.r6);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r5,1),Load(mem,s.r2)),s.r5);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r4,1),Load(mem,s.r0)),s.r4);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(1)),Load(mem,s.r5)),s.r0);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r2.AsInt64(),1).AsUInt64(),Load(mem,s.r4)),s.r2);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(33)),Load(mem,s.r3)),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P18R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r5,s.r8),34);
s.r5=Sel(active,bf,s.r5);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),2).AsUInt64(),Load(mem,s.r6)),s.r7);
s.r4=Sel(active,Vector256.Subtract(Ror(s.r4,1),Load(mem,s.r7)),s.r4);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,2),Load(mem,s.r4)),s.r0);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),Load(mem,s.r0)),s.r2);
s.r1=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r1,2),Load(mem,s.r5)),s.r1);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(33)),Load(mem,s.r2)),s.r3);
s.r6=Sel(active,Vector256.Multiply(Vector256.Xor(s.r6,V(33)),Load(mem,s.r3)),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P19R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r2,s.r8),11);
s.r2=Sel(active,bf,s.r2);
s.r6=Sel(active,Vector256.Xor(Ror(s.r6,55),Load(mem,s.r7)),s.r6);
s.r0=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),3).AsUInt64(),Load(mem,s.r3)),s.r0);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r7,1),Load(mem,s.r0)),s.r7);
s.r4=Sel(active,Vector256.Multiply(Vector256.Add(s.r4,V(1)),Load(mem,s.r2)),s.r4);
s.r3=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),3).AsUInt64(),Load(mem,s.r4)),s.r3);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(9)),Load(mem,s.r5)),s.r1);
s.r5=Sel(active,Vector256.Multiply(Vector256.Xor(s.r5,V(33)),Load(mem,s.r1)),s.r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P20R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r0,s.r8),24);
s.r0=Sel(active,bf,s.r0);
s.r2=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r2,1),Load(mem,s.r4)),s.r2);
s.r6=Sel(active,Vector256.Multiply(Vector256.Add(s.r6,V(1)),Load(mem,s.r3)),s.r6);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),Load(mem,s.r7)),s.r1);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,1),Load(mem,s.r2)),s.r4);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(9)),Load(mem,s.r1)),s.r3);
s.r5=Sel(active,Vector256.Xor(Ror(s.r5,42),Load(mem,s.r6)),s.r5);
s.r7=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),3).AsUInt64(),Load(mem,s.r0)),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P21R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r1,s.r8),20);
s.r1=Sel(active,bf,s.r1);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(9)),Load(mem,s.r2)),s.r7);
s.r5=Sel(active,Vector256.Subtract(Ror(s.r5,44),Load(mem,s.r6)),s.r5);
s.r2=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r2.AsInt64(),3).AsUInt64(),Load(mem,s.r5)),s.r2);
s.r4=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),2).AsUInt64(),Load(mem,s.r1)),s.r4);
s.r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),3).AsUInt64(),Load(mem,s.r3)),s.r0);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(33)),Load(mem,s.r0)),s.r3);
s.r6=Sel(active,Vector256.Multiply(Vector256.Add(s.r6,V(9)),Load(mem,s.r4)),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P22R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r7,s.r8),22);
s.r7=Sel(active,bf,s.r7);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),2).AsUInt64(),Load(mem,s.r0)),s.r5);
s.r6=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r6,2),Load(mem,s.r4)),s.r6);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),Load(mem,s.r6)),s.r2);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(9)),Load(mem,s.r1)),s.r0);
s.r3=Sel(active,Vector256.Xor(Ror(s.r3,26),Load(mem,s.r7)),s.r3);
s.r4=Sel(active,Vector256.Add(Ror(s.r4,1),Load(mem,s.r3)),s.r4);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(33)),Load(mem,s.r5)),s.r1);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P23R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r7,s.r8),59);
s.r7=Sel(active,bf,s.r7);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightArithmetic(s.r0.AsInt64(),2).AsUInt64(),Load(mem,s.r1)),s.r0);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r5,1),Load(mem,s.r2)),s.r5);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),Load(mem,s.r0)),s.r1);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),Load(mem,s.r3)),s.r2);
s.r3=Sel(active,Vector256.Multiply(Vector256.Add(s.r3,V(33)),Load(mem,s.r5)),s.r3);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),2).AsUInt64(),Load(mem,s.r7)),s.r6);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,2),Load(mem,s.r6)),s.r4);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P24R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r7,s.r8),21);
s.r7=Sel(active,bf,s.r7);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),3).AsUInt64(),Load(mem,s.r3)),s.r6);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(33)),Load(mem,s.r6)),s.r1);
s.r5=Sel(active,Vector256.Multiply(Vector256.Add(s.r5,V(9)),Load(mem,s.r1)),s.r5);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,3),Load(mem,s.r7)),s.r0);
s.r4=Sel(active,Vector256.Subtract(Ror(s.r4,31),Load(mem,s.r5)),s.r4);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),Load(mem,s.r4)),s.r2);
s.r3=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r3,2),Load(mem,s.r2)),s.r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P25R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r4,s.r8),33);
s.r4=Sel(active,bf,s.r4);
s.r3=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r3,2),Load(mem,s.r7)),s.r3);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,3),Load(mem,s.r3)),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.Xor(s.r1,V(1)),Load(mem,s.r5)),s.r1);
s.r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r6.AsInt64(),3).AsUInt64(),Load(mem,s.r4)),s.r6);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(1)),Load(mem,s.r2)),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),Load(mem,s.r1)),s.r2);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r5,2),Load(mem,s.r6)),s.r5);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P26R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r3,s.r8),4);
s.r3=Sel(active,bf,s.r3);
s.r4=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r4,2),Load(mem,s.r5)),s.r4);
s.r5=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r5,3),Load(mem,s.r4)),s.r5);
s.r7=Sel(active,Vector256.Multiply(Vector256.Xor(s.r7,V(1)),Load(mem,s.r3)),s.r7);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(9)),Load(mem,s.r1)),s.r2);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(9)),Load(mem,s.r2)),s.r1);
s.r0=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r0,2),Load(mem,s.r6)),s.r0);
s.r6=Sel(active,Vector256.Xor(Ror(s.r6,22),Load(mem,s.r0)),s.r6);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P27R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r4,s.r8),3);
s.r4=Sel(active,bf,s.r4);
s.r6=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r6,1),Load(mem,s.r1)),s.r6);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(1)),Load(mem,s.r4)),s.r1);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(33)),Load(mem,s.r5)),s.r0);
s.r3=Sel(active,Vector256.Subtract(Ror(s.r3,19),Load(mem,s.r0)),s.r3);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r7,3),Load(mem,s.r2)),s.r7);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),1).AsUInt64(),Load(mem,s.r6)),s.r5);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(9)),Load(mem,s.r7)),s.r2);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P28R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r6,s.r8),3);
s.r6=Sel(active,bf,s.r6);
s.r0=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r0,V(9)),Load(mem,s.r3)),s.r0);
s.r5=Sel(active,Vector256.Multiply(Vector256.Add(s.r5,V(33)),Load(mem,s.r7)),s.r5);
s.r3=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r3,V(9)),Load(mem,s.r2)),s.r3);
s.r2=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r2,3),Load(mem,s.r0)),s.r2);
s.r1=Sel(active,Vector256.Add(Vector256.ShiftRightLogical(s.r1,3),Load(mem,s.r5)),s.r1);
s.r4=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r4.AsInt64(),2).AsUInt64(),Load(mem,s.r6)),s.r4);
s.r7=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r7.AsInt64(),3).AsUInt64(),Load(mem,s.r4)),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P29R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r0,s.r8),29);
s.r0=Sel(active,bf,s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r1,V(9)),Load(mem,s.r0)),s.r1);
s.r4=Sel(active,Vector256.Multiply(Vector256.Xor(s.r4,V(9)),Load(mem,s.r5)),s.r4);
s.r5=Sel(active,Vector256.Xor(Ror(s.r5,47),Load(mem,s.r4)),s.r5);
s.r3=Sel(active,Vector256.Add(Ror(s.r3,13),Load(mem,s.r2)),s.r3);
s.r6=Sel(active,Vector256.Subtract(Ror(s.r6,51),Load(mem,s.r3)),s.r6);
s.r2=Sel(active,Vector256.Multiply(Vector256.Xor(s.r2,V(33)),Load(mem,s.r7)),s.r2);
s.r7=Sel(active,Vector256.Xor(Ror(s.r7,31),Load(mem,s.r1)),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P30R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r6,s.r8),52);
s.r6=Sel(active,bf,s.r6);
s.r3=Sel(active,Vector256.Xor(Ror(s.r3,39),Load(mem,s.r7)),s.r3);
s.r4=Sel(active,Vector256.Subtract(Ror(s.r4,56),Load(mem,s.r3)),s.r4);
s.r5=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),3).AsUInt64(),Load(mem,s.r4)),s.r5);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(1)),Load(mem,s.r5)),s.r2);
s.r0=Sel(active,Vector256.Subtract(Vector256.ShiftRightLogical(s.r0,3),Load(mem,s.r6)),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.Add(s.r1,V(33)),Load(mem,s.r2)),s.r1);
s.r7=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r7,V(1)),Load(mem,s.r1)),s.r7);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
take=Vector256.BitwiseAnd(take,Vector256.Equals(Vector256.BitwiseAnd(bf,V(32)),ZERO));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)] static void P31R(ref S s,ref Vector256<ulong> bc,ref Vector256<ulong> sp,nint mem){
var active=ALL;
while(true){
var bf=Ror(Vector256.Multiply(s.r7,s.r8),5);
s.r7=Sel(active,bf,s.r7);
s.r6=Sel(active,Vector256.Xor(Vector256.ShiftRightLogical(s.r6,2),Load(mem,s.r1)),s.r6);
s.r4=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r4,V(9)),Load(mem,s.r5)),s.r4);
s.r2=Sel(active,Vector256.Multiply(Vector256.BitwiseOr(s.r2,V(1)),Load(mem,s.r0)),s.r2);
s.r5=Sel(active,Vector256.Xor(Vector256.ShiftRightArithmetic(s.r5.AsInt64(),3).AsUInt64(),Load(mem,s.r2)),s.r5);
s.r0=Sel(active,Vector256.Xor(Ror(s.r0,62),Load(mem,s.r4)),s.r0);
s.r1=Sel(active,Vector256.Multiply(Vector256.Add(s.r1,V(1)),Load(mem,s.r6)),s.r1);
s.r3=Sel(active,Vector256.Subtract(Vector256.ShiftRightArithmetic(s.r3.AsInt64(),2).AsUInt64(),Load(mem,s.r7)),s.r3);
var take=Vector256.BitwiseAnd(active,NotZero(bc));
bc=Sel(take,Vector256.Subtract(bc,ONE),bc);
if(None(take))break;
active=take;
}
}
[MethodImpl(MethodImplOptions.NoInlining)]public static unsafe Vector256<ulong> Hash4(ulong input,nint mem){var salt=Vector256.Create(input,input+1,input+2,input+3);var k0=V(K0);var k1=V(K1);var a=Vector256.Xor(V(0x736f6d6570736575),k0);var b=Vector256.Xor(V(0x646f72616e646f6d),k1);var c=Vector256.Xor(V(0x6c7967656e657261),k0);var d=Vector256.Xor(V(0x7465646279746573),k1);d=Vector256.Xor(d,salt);Sip(ref a,ref b,ref c,ref d);a=Vector256.Xor(a,salt);c=Vector256.Xor(c,V(0xbb));Sip(ref a,ref b,ref c,ref d);Sip(ref a,ref b,ref c,ref d);Sip(ref a,ref b,ref c,ref d);S s=new(){r0=d,r1=c,r2=b,r3=a};a=Vector256.Xor(a,k0);b=Vector256.Xor(b,k1);c=Vector256.Xor(c,k0);d=Vector256.Xor(d,k1);Sip(ref a,ref b,ref c,ref d);Sip(ref a,ref b,ref c,ref d);Sip(ref a,ref b,ref c,ref d);Sip(ref a,ref b,ref c,ref d);s.r4=d;s.r5=c;s.r6=b;s.r7=a;s.r8=Vector256.BitwiseOr(Vector256.BitwiseAnd(Vector256.Xor(s.r0,s.r4),V(~7UL)),V(3));for(int l=0;l<4;l++)Unsafe.WriteUnaligned((byte*)mem+l*16392+16384,s.r8[l]);var sp=V(16384);for(int rep=0;rep<4;rep++){var bc=V(32);P0W(ref s,ref bc,ref sp,mem);
P1W(ref s,ref bc,ref sp,mem);
P2W(ref s,ref bc,ref sp,mem);
P3W(ref s,ref bc,ref sp,mem);
P4W(ref s,ref bc,ref sp,mem);
P5W(ref s,ref bc,ref sp,mem);
P6W(ref s,ref bc,ref sp,mem);
P7W(ref s,ref bc,ref sp,mem);
P8W(ref s,ref bc,ref sp,mem);
P9W(ref s,ref bc,ref sp,mem);
P10W(ref s,ref bc,ref sp,mem);
P11W(ref s,ref bc,ref sp,mem);
P12W(ref s,ref bc,ref sp,mem);
P13W(ref s,ref bc,ref sp,mem);
P14W(ref s,ref bc,ref sp,mem);
P15W(ref s,ref bc,ref sp,mem);
P16W(ref s,ref bc,ref sp,mem);
P17W(ref s,ref bc,ref sp,mem);
P18W(ref s,ref bc,ref sp,mem);
P19W(ref s,ref bc,ref sp,mem);
P20W(ref s,ref bc,ref sp,mem);
P21W(ref s,ref bc,ref sp,mem);
P22W(ref s,ref bc,ref sp,mem);
P23W(ref s,ref bc,ref sp,mem);
P24W(ref s,ref bc,ref sp,mem);
P25W(ref s,ref bc,ref sp,mem);
P26W(ref s,ref bc,ref sp,mem);
P27W(ref s,ref bc,ref sp,mem);
P28W(ref s,ref bc,ref sp,mem);
P29W(ref s,ref bc,ref sp,mem);
P30W(ref s,ref bc,ref sp,mem);
P31W(ref s,ref bc,ref sp,mem);}for(int rep=0;rep<4;rep++){var bc=V(32);P0R(ref s,ref bc,ref sp,mem);
P1R(ref s,ref bc,ref sp,mem);
P2R(ref s,ref bc,ref sp,mem);
P3R(ref s,ref bc,ref sp,mem);
P4R(ref s,ref bc,ref sp,mem);
P5R(ref s,ref bc,ref sp,mem);
P6R(ref s,ref bc,ref sp,mem);
P7R(ref s,ref bc,ref sp,mem);
P8R(ref s,ref bc,ref sp,mem);
P9R(ref s,ref bc,ref sp,mem);
P10R(ref s,ref bc,ref sp,mem);
P11R(ref s,ref bc,ref sp,mem);
P12R(ref s,ref bc,ref sp,mem);
P13R(ref s,ref bc,ref sp,mem);
P14R(ref s,ref bc,ref sp,mem);
P15R(ref s,ref bc,ref sp,mem);
P16R(ref s,ref bc,ref sp,mem);
P17R(ref s,ref bc,ref sp,mem);
P18R(ref s,ref bc,ref sp,mem);
P19R(ref s,ref bc,ref sp,mem);
P20R(ref s,ref bc,ref sp,mem);
P21R(ref s,ref bc,ref sp,mem);
P22R(ref s,ref bc,ref sp,mem);
P23R(ref s,ref bc,ref sp,mem);
P24R(ref s,ref bc,ref sp,mem);
P25R(ref s,ref bc,ref sp,mem);
P26R(ref s,ref bc,ref sp,mem);
P27R(ref s,ref bc,ref sp,mem);
P28R(ref s,ref bc,ref sp,mem);
P29R(ref s,ref bc,ref sp,mem);
P30R(ref s,ref bc,ref sp,mem);
P31R(ref s,ref bc,ref sp,mem);}a=s.r0;b=s.r1;c=s.r2;d=s.r3;Sip(ref a,ref b,ref c,ref d);var o3=d;a=s.r4;b=s.r5;c=s.r6;d=s.r7;Sip(ref a,ref b,ref c,ref d);return Vector256.Xor(Vector256.Xor(o3,d),s.r8);}
}
class Program{static unsafe void Main(){nint mem=(nint)NativeMemory.Alloc((nuint)(4*16392));var q=HW.Hash4(123456,mem);Console.WriteLine($"CHECK {q[0]:x16} {q[1]:x16} {q[2]:x16} {q[3]:x16}");for(int i=0;i<2000;i++)_=HW.Hash4((ulong)(i*4),mem);const int N=200000;ulong sink=0;var sw=Stopwatch.StartNew();for(int i=0;i<N/4;i++){q=HW.Hash4((ulong)(i*4),mem);sink^=q[0]^q[1]^q[2]^q[3];}sw.Stop();Console.WriteLine($"RATE {N/sw.Elapsed.TotalSeconds:F1} sink={sink:x16}");NativeMemory.Free((void*)mem);}}
