using System.Diagnostics;
using System.Globalization;
using System.Numerics;
using Museum.ChallengeSha256;

internal static class Program
{
    private static int Count(string raw, int minimum)
    {
        int value = int.Parse(raw, NumberStyles.None, CultureInfo.InvariantCulture);
        if (value < minimum || value > Sha256Kernel.CandidatesPerRow * Sha256Kernel.MaxRows)
            throw new ArgumentOutOfRangeException(nameof(raw), value, "invalid nonce count");
        return value;
    }

    private static ulong Run(string mode, int count) => mode switch
    {
        "portable" => Sha256Kernel.RunVector(count),
        "optimized" => Vector<uint>.Count == 16
            ? Sha256Kernel.RunVectorOptimized(count) : Sha256Kernel.RunVector(count),
        "match-optimized" => Vector<uint>.Count == 16
            ? Sha256Kernel.RunVectorMatchOptimized(count) : Sha256Kernel.RunVectorMatch(count),
        "match" => Sha256Kernel.RunVectorMatch(count),
        "scalar" => Sha256Kernel.RunScalar(count),
        "bcl" => Sha256Kernel.RunBcl(count),
        _ => throw new ArgumentException($"unknown backend: {mode}")
    };

    private static int Main(string[] args)
    {
        try
        {
            if (args.Length == 1 && args[0] == "--self-test")
            {
                Sha256Kernel.SelfTest();
                Console.WriteLine($"PASS portable SIMD lanes={Vector<uint>.Count}");
                return 0;
            }
            if (args.Length != 5 || args[0] != "--mode")
                throw new ArgumentException("usage: ChallengeSha256 --mode portable|optimized|match|match-optimized|scalar|bcl WARMUP ITERATIONS SAMPLES OR --self-test");

            string mode = args[1];
            int warmup = Count(args[2], 0);
            int iterations = Count(args[3], 1);
            int samples = Count(args[4], 1);
            Console.WriteLine($"BENCHMARK_DIMENSIONS v1 {warmup} {iterations} {samples}");
            Sha256Kernel.SelfTest();
            Console.WriteLine($"ENGINE dotnet MODE {mode} VECTOR_LANES {Vector<uint>.Count}");
            Console.WriteLine($"CHECK {Sha256Kernel.Check()}");
            _ = Run(mode, warmup);

            for (int sample = 0; sample < samples; sample++)
            {
                long start = Stopwatch.GetTimestamp();
                ulong sink = Run(mode, iterations);
                long end = Stopwatch.GetTimestamp();
                double seconds = (end - start) / (double)Stopwatch.Frequency;
                double rate = iterations / seconds;
                Console.WriteLine($"RATE {rate.ToString("F6", CultureInfo.InvariantCulture)} sink={sink:x16}");
            }
            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine(error);
            return 1;
        }
    }
}
