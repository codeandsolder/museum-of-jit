using System;
using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.Intrinsics;
class Program {
 const ulong M=0xd1b54a32d192ed03UL, X=0x9e3779b97f4a7c15UL;
 [MethodImpl(MethodImplOptions.NoInlining)] static ulong Scalar8(ulong s){ulong a=s+1,b=s+2,c=s+3,d=s+4,e=s+5,f=s+6,g=s+7,h=s+8;for(int i=0;i<32;i++){a=BitOperations.RotateRight(a*M,17)^X;b=BitOperations.RotateRight(b*M,17)^X;c=BitOperations.RotateRight(c*M,17)^X;d=BitOperations.RotateRight(d*M,17)^X;e=BitOperations.RotateRight(e*M,17)^X;f=BitOperations.RotateRight(f*M,17)^X;g=BitOperations.RotateRight(g*M,17)^X;h=BitOperations.RotateRight(h*M,17)^X;}return a^b^c^d^e^f^g^h;}
 [MethodImpl(MethodImplOptions.NoInlining)] static ulong V256(ulong s){var v=Vector256.Create(s+1,s+2,s+3,s+4);var m=Vector256.Create(M);var x=Vector256.Create(X);for(int i=0;i<32;i++){v=Vector256.Multiply(v,m);v=Vector256.BitwiseOr(Vector256.ShiftRightLogical(v,17),Vector256.ShiftLeft(v,47));v=Vector256.Xor(v,x);}return v[0]^v[1]^v[2]^v[3];}
 [MethodImpl(MethodImplOptions.NoInlining)] static ulong V512(ulong s){var v=Vector512.Create(s+1,s+2,s+3,s+4,s+5,s+6,s+7,s+8);var m=Vector512.Create(M);var x=Vector512.Create(X);for(int i=0;i<32;i++){v=Vector512.Multiply(v,m);v=Vector512.BitwiseOr(Vector512.ShiftRightLogical(v,17),Vector512.ShiftLeft(v,47));v=Vector512.Xor(v,x);}ulong z=0;for(int i=0;i<8;i++)z^=v[i];return z;}
 static void Bench(string n,int lanes,Func<ulong,ulong> f){for(int i=0;i<20000;i++)_=f((ulong)i);const int C=500000;ulong sink=0;var sw=Stopwatch.StartNew();for(int i=0;i<C;i++)sink^=f((ulong)i);sw.Stop();Console.WriteLine($"{n} {C*lanes/sw.Elapsed.TotalSeconds/1e6:F2} M-lanes/s {sw.Elapsed.TotalSeconds:F3}s sink={sink:x}");}
 static void Main(){Console.WriteLine($"V256={Vector256.IsHardwareAccelerated} V512={Vector512.IsHardwareAccelerated}");Bench("scalar8",8,Scalar8);Bench("v256",4,V256);Bench("v512",8,V512);}
}
