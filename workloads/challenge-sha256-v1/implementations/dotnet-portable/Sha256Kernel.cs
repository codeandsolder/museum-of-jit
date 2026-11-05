using System.Buffers.Binary;
using System.Numerics;
using System.Runtime.CompilerServices;
using System.Security.Cryptography;
using System.Text;

namespace Museum.ChallengeSha256;

/// <summary>
/// A one-block SHA-256 compression engine over prefix[42] || ASCII_DECIMAL(nonce)[9].
/// Compatible with challenge-solvers::SingleBlockMessage::new(prefix, 0):
/// prefix length 42 deliberately exercises its aligned seven-digit inner loop.
/// No ISA-specific intrinsics, unsafe code, native dependencies, or dynamic codegen.
/// </summary>
internal static partial class Sha256Kernel
{
    internal const int PrefixLength = 42;
    internal const int CandidateLength = PrefixLength + 9;
    internal const int CandidatesPerRow = 16;
    internal const int MaxRows = 10_000_000;
    private const uint DecimalScale = 10_000_000;

    internal static readonly byte[] Prefix = MakePrefix();
    private static readonly uint[] Initial =
    [
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    ];
    private static readonly uint[] Round =
    [
        0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u,
        0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
        0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u,
        0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
        0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu,
        0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
        0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u,
        0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
        0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u,
        0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
        0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u,
        0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
        0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u,
        0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
        0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u,
        0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
    ];

    internal static readonly uint[] Template = MakeTemplate();
    private static readonly uint[] HotState = MakeHotState();
    private static readonly Vector<uint>[] VectorHotState = Array.ConvertAll(HotState, value => new Vector<uint>(value));
    private static readonly Vector<uint>[] VectorRound = Array.ConvertAll(Round, value => new Vector<uint>(value));
    private static readonly Vector<uint>[] VectorInitial = Array.ConvertAll(Initial, value => new Vector<uint>(value));

    private static byte[] MakePrefix()
    {
        var prefix = new byte[PrefixLength];
        prefix.AsSpan().Fill((byte)'x');
        "challenge-disasterer|sha256|"u8.CopyTo(prefix);
        return prefix;
    }

    private static uint[] MakeTemplate()
    {
        Span<byte> bytes = stackalloc byte[64];
        Prefix.CopyTo(bytes);
        bytes[CandidateLength] = 0x80;
        BinaryPrimitives.WriteUInt64BigEndian(bytes[56..], CandidateLength * 8UL);
        var words = new uint[16];
        for (int i = 0; i < words.Length; i++)
            words[i] = BinaryPrimitives.ReadUInt32BigEndian(bytes[(i * 4)..]);
        return words;
    }

    // SHA rounds 0..9 cannot depend on nonce words 10, 11, or 12. Running
    // those rounds once outside the hot loop exactly mirrors the Rust solver's
    // prefix hotstart while keeping the implementation architecture-neutral.
    private static uint[] MakeHotState()
    {
        uint a = Initial[0], b = Initial[1], c = Initial[2], d = Initial[3];
        uint e = Initial[4], f = Initial[5], g = Initial[6], h = Initial[7];
        unchecked
        {
            for (int i = 0; i < 10; i++)
            {
                uint t1 = h + B1(e) + ((e & f) ^ (~e & g)) + Round[i] + Template[i];
                uint t2 = B0(a) + ((a & b) ^ (a & c) ^ (b & c));
                h = g; g = f; f = e; e = d + t1;
                d = c; c = b; b = a; a = t1 + t2;
            }
        }
        return [a, b, c, d, e, f, g, h];
    }

    // Two ASCII digits occupy bytes 42 and 43. Every other word is lane-invariant.
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    internal static uint PrefixWord(int laneId) =>
        (Template[10] & 0xffff0000u) |
        ((uint)('0' + laneId / 10) << 8) |
        (uint)('0' + laneId % 10);

    // Seven ASCII digits occupy bytes 44..50, with the 0x80 pad in byte 51.
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    internal static void InnerWords(int inner, out uint word11, out uint word12)
    {
        uint n = (uint)inner;
        uint d6 = n % 10; n /= 10;
        uint d5 = n % 10; n /= 10;
        uint d4 = n % 10; n /= 10;
        uint d3 = n % 10; n /= 10;
        uint d2 = n % 10; n /= 10;
        uint d1 = n % 10; n /= 10;
        word11 = ((n + (uint)'0') << 24) | ((d1 + (uint)'0') << 16) |
                 ((d2 + (uint)'0') << 8) | (d3 + (uint)'0');
        word12 = ((d4 + (uint)'0') << 24) | ((d5 + (uint)'0') << 16) |
                 ((d6 + (uint)'0') << 8) | 0x80u;
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static uint S0(uint v) => BitOperations.RotateRight(v, 7) ^ BitOperations.RotateRight(v, 18) ^ (v >> 3);
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static uint S1(uint v) => BitOperations.RotateRight(v, 17) ^ BitOperations.RotateRight(v, 19) ^ (v >> 10);
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static uint B0(uint v) => BitOperations.RotateRight(v, 2) ^ BitOperations.RotateRight(v, 13) ^ BitOperations.RotateRight(v, 22);
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static uint B1(uint v) => BitOperations.RotateRight(v, 6) ^ BitOperations.RotateRight(v, 11) ^ BitOperations.RotateRight(v, 25);

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static Vector<uint> S0(Vector<uint> v) => ((v >> 7) | (v << 25)) ^ ((v >> 18) | (v << 14)) ^ (v >> 3);
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static Vector<uint> S1(Vector<uint> v) => ((v >> 17) | (v << 15)) ^ ((v >> 19) | (v << 13)) ^ (v >> 10);
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static Vector<uint> B0(Vector<uint> v) => ((v >> 2) | (v << 30)) ^ ((v >> 13) | (v << 19)) ^ ((v >> 22) | (v << 10));
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static Vector<uint> B1(Vector<uint> v) => ((v >> 6) | (v << 26)) ^ ((v >> 11) | (v << 21)) ^ ((v >> 25) | (v << 7));

    /// <summary>Scalar integer oracle, no platform cryptographic acceleration.</summary>
    internal static void CompressScalar(ReadOnlySpan<uint> input, Span<uint> digest)
    {
        Span<uint> w = stackalloc uint[16];
        input.CopyTo(w);
        uint a = HotState[0], b = HotState[1], c = HotState[2], d = HotState[3];
        uint e = HotState[4], f = HotState[5], g = HotState[6], h = HotState[7];
        unchecked
        {
            for (int i = 10; i < 64; i++)
            {
                uint schedule;
                if (i < 16) schedule = w[i];
                else
                {
                    int j = i & 15;
                    schedule = w[j] + S0(w[(i - 15) & 15]) + w[(i - 7) & 15] + S1(w[(i - 2) & 15]);
                    w[j] = schedule;
                }
                uint t1 = h + B1(e) + ((e & f) ^ (~e & g)) + Round[i] + schedule;
                uint t2 = B0(a) + ((a & b) ^ (a & c) ^ (b & c));
                h = g; g = f; f = e; e = d + t1;
                d = c; c = b; b = a; a = t1 + t2;
            }
            digest[0] = Initial[0] + a; digest[1] = Initial[1] + b;
            digest[2] = Initial[2] + c; digest[3] = Initial[3] + d;
            digest[4] = Initial[4] + e; digest[5] = Initial[5] + f;
            digest[6] = Initial[6] + g; digest[7] = Initial[7] + h;
        }
    }

    /// <summary>
    /// Portable System.Numerics SIMD: RyuJIT chooses the supported native vector width,
    /// including a software fallback. No x86/ARM feature tests or HWIntrinsic calls.
    /// </summary>
    internal static void CompressVector(ReadOnlySpan<Vector<uint>> input, Span<Vector<uint>> digest)
    {
        Span<Vector<uint>> w = stackalloc Vector<uint>[16];
        input.CopyTo(w);
        var a = VectorHotState[0]; var b = VectorHotState[1];
        var c = VectorHotState[2]; var d = VectorHotState[3];
        var e = VectorHotState[4]; var f = VectorHotState[5];
        var g = VectorHotState[6]; var h = VectorHotState[7];

        for (int i = 10; i < 64; i++)
        {
            Vector<uint> schedule;
            if (i < 16) schedule = w[i];
            else
            {
                int j = i & 15;
                schedule = w[j] + S0(w[(i - 15) & 15]) + w[(i - 7) & 15] + S1(w[(i - 2) & 15]);
                w[j] = schedule;
            }
            var t1 = h + B1(e) + ((e & f) ^ (~e & g)) + VectorRound[i] + schedule;
            var t2 = B0(a) + ((a & b) ^ (a & c) ^ (b & c));
            h = g; g = f; f = e; e = d + t1;
            d = c; c = b; b = a; a = t1 + t2;
        }
        digest[0] = VectorInitial[0] + a; digest[1] = VectorInitial[1] + b;
        digest[2] = VectorInitial[2] + c; digest[3] = VectorInitial[3] + d;
        digest[4] = VectorInitial[4] + e; digest[5] = VectorInitial[5] + f;
        digest[6] = VectorInitial[6] + g; digest[7] = VectorInitial[7] + h;
    }

    /// <summary>Generate an independent SHA-256 oracle, including the exact decimal nonce formatting.</summary>
    internal static void HashBcl(int lane, int inner, Span<byte> digest)
    {
        Span<byte> message = stackalloc byte[CandidateLength];
        Prefix.CopyTo(message);
        uint nonce = (uint)lane * DecimalScale + (uint)inner;
        for (int i = CandidateLength - 1; i >= PrefixLength; i--)
        {
            message[i] = (byte)('0' + nonce % 10);
            nonce /= 10;
        }
        SHA256.HashData(message, digest);
    }

    internal static string Check()
    {
        Span<byte> digest = stackalloc byte[32];
        HashBcl(10, 0, digest);
        return Convert.ToHexStringLower(digest[..8]);
    }

    // The enumeration follows challenge-solvers' native-width search layout:
    // one set of lane prefixes is exhausted over inner=0..9,999,999 before
    // proceeding to the next prefix set. All .NET modes share this same order.
    internal static ulong RunScalar(int count)
    {
        Span<uint> block = stackalloc uint[16];
        Span<uint> state = stackalloc uint[8];
        Template.CopyTo(block);
        uint sink0 = 0, sink1 = 0;
        int done = 0;
        int width = Vector<uint>.Count;
        for (int group = 0; done < count; group += width)
        {
            for (int inner = 0; inner < MaxRows && done < count; inner++)
            {
                InnerWords(inner, out block[11], out block[12]);
                for (int lane = 0; lane < width && done < count; lane++, done++)
                {
                    block[10] = PrefixWord(10 + group + lane);
                    CompressScalar(block, state);
                    sink0 ^= state[0]; sink1 ^= state[1];
                }
            }
        }
        return ((ulong)sink0 << 32) | sink1;
    }

    internal static ulong RunBcl(int count)
    {
        Span<byte> digest = stackalloc byte[32];
        uint sink0 = 0, sink1 = 0;
        int width = Vector<uint>.Count;
        int done = 0;
        for (int group = 0; done < count; group += width)
        {
            for (int inner = 0; inner < MaxRows && done < count; inner++)
            {
                for (int lane = 0; lane < width && done < count; lane++, done++)
                {
                    HashBcl(10 + group + lane, inner, digest);
                    sink0 ^= BinaryPrimitives.ReadUInt32BigEndian(digest);
                    sink1 ^= BinaryPrimitives.ReadUInt32BigEndian(digest[4..]);
                }
            }
        }
        return ((ulong)sink0 << 32) | sink1;
    }

    internal static ulong RunVector(int count)
    {
        int width = Vector<uint>.Count;
        if (width > CandidatesPerRow || CandidatesPerRow % width != 0)
            throw new NotSupportedException("Vector<uint>.Count must divide 16");

        Span<Vector<uint>> block = stackalloc Vector<uint>[16];
        Span<Vector<uint>> state = stackalloc Vector<uint>[8];
        for (int i = 0; i < 16; i++) block[i] = new Vector<uint>(Template[i]);
        Span<uint> words = stackalloc uint[width];
        Span<uint> partial0 = stackalloc uint[width];
        Span<uint> partial1 = stackalloc uint[width];
        var sink0 = Vector<uint>.Zero;
        var sink1 = Vector<uint>.Zero;
        uint tail0 = 0, tail1 = 0;
        int done = 0;
        for (int group = 0; done < count; group += width)
        {
            for (int lane = 0; lane < width; lane++)
                words[lane] = PrefixWord(10 + group + lane);
            block[10] = new Vector<uint>(words);
            for (int inner = 0; inner < MaxRows && done < count; inner++)
            {
                InnerWords(inner, out uint word11, out uint word12);
                block[11] = new Vector<uint>(word11);
                block[12] = new Vector<uint>(word12);
                CompressVector(block, state);
                int valid = Math.Min(width, count - done);
                if (valid == width)
                {
                    sink0 ^= state[0]; sink1 ^= state[1];
                }
                else
                {
                    state[0].CopyTo(partial0);
                    state[1].CopyTo(partial1);
                    for (int lane = 0; lane < valid; lane++)
                    {
                        tail0 ^= partial0[lane];
                        tail1 ^= partial1[lane];
                    }
                }
                done += valid;
            }
        }
        sink0.CopyTo(partial0);
        sink1.CopyTo(partial1);
        for (int lane = 0; lane < width; lane++)
        {
            tail0 ^= partial0[lane];
            tail1 ^= partial1[lane];
        }
        return ((ulong)tail0 << 32) | tail1;
    }

    // Same 64-bit equality target and "no hit" accounting used by the
    // production Rust solver, with no per-candidate digest XOR/materialization.
    // This is a distinct benchmark mode: never conflate its sink with RunVector.
    internal static ulong RunVectorMatch(int count)
    {
        int width = Vector<uint>.Count;
        if (width > CandidatesPerRow || CandidatesPerRow % width != 0)
            throw new NotSupportedException("Vector<uint>.Count must divide 16");

        Span<Vector<uint>> block = stackalloc Vector<uint>[16];
        Span<Vector<uint>> state = stackalloc Vector<uint>[8];
        for (int i = 0; i < 16; i++) block[i] = new Vector<uint>(Template[i]);
        Span<uint> words = stackalloc uint[width];
        Span<uint> partialMask = stackalloc uint[width];
        var targetHi = new Vector<uint>(0xbaddcafeu);
        var targetLo = new Vector<uint>(0x12345678u);
        var allBits = new Vector<uint>(uint.MaxValue);

        int done = 0;
        for (int group = 0; done < count; group += width)
        {
            for (int lane = 0; lane < width; lane++)
                words[lane] = PrefixWord(10 + group + lane);
            block[10] = new Vector<uint>(words);

            for (int inner = 0; inner < MaxRows && done < count; inner++)
            {
                InnerWords(inner, out uint word11, out uint word12);
                block[11] = new Vector<uint>(word11);
                block[12] = new Vector<uint>(word12);
                CompressVector(block, state);
                int valid = Math.Min(width, count - done);
                var matches = Vector.Equals(state[0], targetHi) & Vector.Equals(state[1], targetLo);
                if (Vector.EqualsAny(matches, allBits))
                {
                    matches.CopyTo(partialMask);
                    for (int lane = 0; lane < valid; lane++)
                        if (partialMask[lane] == uint.MaxValue)
                            throw new InvalidOperationException("unexpected match in fixed-target benchmark");
                }
                done += valid;
            }
        }

        return (ulong)done;
    }

    internal static void SelfTest()
    {
        // Every lane and multiple carries through the 7-digit decimal field.
        int[] cases = [0, 1, 9, 10, 99, 100, 999, 1000, 999999, 1000000, 9999999];
        Span<uint> block = stackalloc uint[16];
        Span<uint> scalar = stackalloc uint[8];
        Span<byte> bcl = stackalloc byte[32];
        Template.CopyTo(block);
        foreach (int inner in cases)
        {
            InnerWords(inner, out block[11], out block[12]);
            for (int lane = 10; lane < 26; lane++)
            {
                block[10] = PrefixWord(lane);
                HashBcl(lane, inner, bcl);
                CompressScalar(block, scalar);
                for (int word = 0; word < 8; word++)
                {
                    uint expected = BinaryPrimitives.ReadUInt32BigEndian(bcl[(word * 4)..]);
                    if (scalar[word] != expected)
                        throw new InvalidOperationException($"scalar mismatch lane={lane} inner={inner} word={word}");
                }
            }
        }

        int width = Vector<uint>.Count;
        Span<Vector<uint>> vblock = stackalloc Vector<uint>[16];
        Span<Vector<uint>> vstate = stackalloc Vector<uint>[8];
        Span<Vector<uint>> optimizedState = stackalloc Vector<uint>[8];
        Span<uint> laneWords = stackalloc uint[width];
        for (int i = 0; i < 16; i++) vblock[i] = new Vector<uint>(Template[i]);
        foreach (int inner in cases)
        {
            InnerWords(inner, out uint word11, out uint word12);
            vblock[11] = new Vector<uint>(word11);
            vblock[12] = new Vector<uint>(word12);
            for (int group = 0; group < 16; group += width)
            {
                for (int lane = 0; lane < width; lane++)
                    laneWords[lane] = PrefixWord(10 + group + lane);
                vblock[10] = new Vector<uint>(laneWords);
                CompressVector(vblock, vstate);
                CompressVectorUnrolled(vblock, optimizedState);
                for (int lane = 0; lane < width; lane++)
                {
                    HashBcl(10 + group + lane, inner, bcl);
                    for (int word = 0; word < 8; word++)
                    {
                        uint expected = BinaryPrimitives.ReadUInt32BigEndian(bcl[(word * 4)..]);
                        if (vstate[word][lane] != expected ||
                            optimizedState[word][lane] != expected)
                            throw new InvalidOperationException($"vector mismatch lane={group + lane} inner={inner} word={word}");
                    }
                }
            }
        }
        foreach (int count in new[] { 1, 3, 7, 15, 16, 17, 31, 32, 63, 129, 1024 })
        {
            ulong expected = RunBcl(count);
            if (RunScalar(count) != expected || RunVector(count) != expected ||
                RunVectorOptimized(count) != expected ||
                RunVectorMatch(count) != (ulong)count ||
                RunVectorMatchOptimized(count) != (ulong)count)
                throw new InvalidOperationException($"partial batch mismatch count={count}");
        }
    }
}
