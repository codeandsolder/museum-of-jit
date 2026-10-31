using System; using System.Diagnostics; using System.Reflection.Emit; using System.Linq;
class Program {
 const int NOPS=1024;
 static Func<ulong,ulong> Gen(){var d=new DynamicMethod("g",typeof(ulong),new[]{typeof(ulong)});var il=d.GetILGenerator();il.Emit(OpCodes.Ldarg_0);for(int i=0;i<NOPS;i++){il.Emit(OpCodes.Ldc_I8,1L);il.Emit(OpCodes.Add);}il.Emit(OpCodes.Ret);return (Func<ulong,ulong>)d.CreateDelegate(typeof(Func<ulong,ulong>));}
 static readonly byte[] Code=MakeCode();
 static byte[] MakeCode(){var b=new byte[1+NOPS*10+1];int p=0;b[p++]=0x02;for(int i=0;i<NOPS;i++){b[p++]=0x21;BitConverter.GetBytes(1L).CopyTo(b,p);p+=8;b[p++]=0x58;}b[p++]=0x2a;return b;}
 static Func<ulong,ulong> Raw(){var d=new DynamicMethod("r",typeof(ulong),new[]{typeof(ulong)});var x=d.GetDynamicILInfo();x.SetLocalSignature(new byte[]{0x07,0x00});x.SetCode(Code,2);return (Func<ulong,ulong>)d.CreateDelegate(typeof(Func<ulong,ulong>));}
 static void Test(string n,Func<Func<ulong,ulong>> make){double[] t=new double[128];ulong z=0;for(int i=0;i<t.Length;i++){var sw=Stopwatch.StartNew();var f=make();sw.Stop();t[i]=sw.Elapsed.TotalMicroseconds;z^=f((ulong)i);}Array.Sort(t);Console.WriteLine($"{n} max={t[^1]:F1} med={t[t.Length/2]:F1} p90={t[(int)(t.Length*.9)]:F1} min={t[0]:F1} check={z}");}
 static void Main(){Test("gen",Gen);Test("raw",Raw);}
}
