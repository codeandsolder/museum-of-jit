using System;
using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.Intrinsics;
static class Program {
 const int N=5_000_000;
 [MethodImpl(MethodImplOptions.NoInlining)] static ulong Scalar4(ulong seed){
  ulong a=seed+1,b=seed+2,c=seed+3,d=seed+4;
  for(int i=0;i<32;i++){
   a=BitOperations.RotateRight(a*0xd1b54a32d192ed03UL,17)^0x9e3779b97f4a7c15UL;
   b=BitOperations.RotateRight(b*0x94d049bb133111ebUL,17)^0x9e3779b97f4a7c15UL;
   c=BitOperations.RotateRight(c*0x8538ecf5bd456ea3UL,17)^0x9e3779b97f4a7c15UL;
   d=BitOperations.RotateRight(d*0xda942042e4dd58b5UL,17)^0x9e3779b97f4a7c15UL;
  } return a^b^c^d;
 }
 [MethodImpl(MethodImplOptions.NoInlining)] static ulong Vec4(ulong seed){
  var x=Vector256.Create(seed+1,seed+2,seed+3,seed+4);
  var m=Vector256.Create(0xd1b54a32d192ed03UL,0x94d049bb133111ebUL,0x8538ecf5bd456ea3UL,0xda942042e4dd58b5UL);
  var k=Vector256.Create(0x9e3779b97f4a7c15UL);
  for(int i=0;i<32;i++){var p=Vector256.Multiply(x,m);x=(Vector256.ShiftRightLogical(p,17)|Vector256.ShiftLeft(p,47))^k;}
  return x.GetElement(0)^x.GetElement(1)^x.GetElement(2)^x.GetElement(3);
 }
 static void Main(){ulong s=0;for(int i=0;i<10000;i++){s^=Scalar4((ulong)i);s^=Vec4((ulong)i);}var sw=Stopwatch.StartNew();for(int i=0;i<N;i++)s^=Scalar4((ulong)i);sw.Stop();Console.WriteLine($"scalar {N*4/sw.Elapsed.TotalSeconds/1e6:F1}");sw.Restart();for(int i=0;i<N;i++)s^=Vec4((ulong)i);sw.Stop();Console.WriteLine($"vec {N*4/sw.Elapsed.TotalSeconds/1e6:F1} sink={s:x}");}
}