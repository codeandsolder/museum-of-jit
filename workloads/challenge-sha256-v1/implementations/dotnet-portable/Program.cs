using System.Diagnostics;
using System.Globalization;
using System.Numerics;
using Museum.ChallengeSha256;

internal static class Program
{
    private static int EnvInteger(string key, int defaultValue, int minimum)
    {
        string? raw = Environment.GetEnvironmentVariable(key);
        int value = raw is null ? defaultValue : int.Parse(raw, CultureInfo.InvariantCulture);
        if (value < minimum || value > Sha256Kernel.CandidatesPerRow * Sha256Kernel.MaxRows)
            throw new ArgumentOutOfRangeException(key, value, "invalid nonce count");
        return value;
    }

    private static ulong Run(string mode, int count) => mode switch
    {
        "portable" => Sha256Kernel.RunVector(count),
        "scalar" => Sha256Kernel.RunScalar(count),
        "bcl" => Sha256Kernel.RunBcl(count),
        _ => throw new ArgumentException($"unknown backend: {mode}")
    };

    private static int Main(string[] args)
    {
        try
        {
            if (args.Contains("--self-test", StringComparer.Ordinal))
            {
                Sha256Kernel.SelfTest();
                Console.WriteLine($"PASS portable SIMD lanes={Vector<uint>.Count}");
                return 0;
            }
            if (args.Length != 2 || args[0] != "--mode")
                throw new ArgumentException("usage: ChallengeSha256 --mode portable|scalar|bcl OR --self-test");

            string mode = args[1];
            Sha256Kernel.SelfTest();
            Console.WriteLine($"ENGINE dotnet MODE {mode} VECTOR_LANES {Vector<uint>.Count}");
            Console.WriteLine($"CHECK {Sha256Kernel.Check()}");
            int warmup = EnvInteger("MUSEUM_WARMUP", 30000, 0);
            int iterations = EnvInteger("MUSEUM_ITERATIONS", 300000, 1);
            int samples = EnvInteger("MUSEUM_SAMPLES", 7, 1);
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
