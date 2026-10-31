using System;
using System.Diagnostics;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Runtime.Intrinsics;

static class Program {
    const int N = 5_000_000;
    [MethodImpl(MethodImplOptions.NoInlining)]
    static ulong Scalar4(ulong seed) {
        ulong a=seed+1,b=seed+2,c=seed+3,d=seed+4;
        for (int i=0;i<32;i++) {
            a=BitOperations.RotateRight(a*b,17) ^ 0x9e3779b97f4a7c15UL;
            b=BitOperations.RotateRight(b*c,23) + 0xd1b54a32d192ed03UL;
            c=BitOperations.RotateRight(c*d,31) ^ 0x94d049bb133111ebUL;
            d=BitOperations.RotateRight(d*a,11) + 0x8538ecf5bd456ea3UL;
        }
        return a^b^c^d;
    }
    [MethodImpl(MethodImplOptions.NoInlining)]
    static ulong Vec(ulong seed) {
        var x=Vector256.Create(seed+1, seed+2, seed+3, seed+4);
        var y=Vector256.Create(seed+5, seed+6, seed+7, seed+8);
        var k=Vector256.Create(0x9e3779b97f4a7c15UL);
        var add=Vector256.Create(0xd1b54a32d192ed03UL);
        for(int i=0;i<32;i++) {
            var m=Vector256.Multiply(x,y);
            x=(Vector256.ShiftRightLogical(m,17) | Vector256.ShiftLeft(m,47)) ^ k;
            y=Vector256.Add(y, add);
        }
        return x.GetElement(0)^x.GetElement(1)^x.GetElement(2)^x.GetElement(3);
    }
    static void Main(){
        Console.WriteLine($"V256={Vector256.IsHardwareAccelerated} VectorULongCount={Vector<ulong>.Count}");
        ulong sink=0;
        for(int i=0;i<10000;i++){sink^=Scalar4((ulong)i);sink^=Vec((ulong)i);} 
        var sw=Stopwatch.StartNew(); for(int i=0;i<N;i++) sink^=Scalar4((ulong)i); sw.Stop(); Console.WriteLine($"scalar4 {N*4/sw.Elapsed.TotalSeconds/1e6:F1} M-lanes/s t={sw.Elapsed.TotalSeconds:F3}");
        sw.Restart(); for(int i=0;i<N;i++) sink^=Vec((ulong)i); sw.Stop(); Console.WriteLine($"vec4 {N*4/sw.Elapsed.TotalSeconds/1e6:F1} M-lanes/s t={sw.Elapsed.TotalSeconds:F3} sink={sink:x}");
    }
}
