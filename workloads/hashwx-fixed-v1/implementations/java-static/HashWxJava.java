import sun.misc.Unsafe;
import java.lang.reflect.Field;
public final class HashWxJava {
    static final Unsafe U;
    static { try { Field f=Unsafe.class.getDeclaredField("theUnsafe"); f.setAccessible(true); U=(Unsafe)f.get(null); } catch(Exception e){ throw new RuntimeException(e);} }
    static long hash(long salt,long mem) {
        long k0=0x1716151413121110L, k1=0x1f1e1d1c1b1a1918L;
        long v0=0x736f6d6570736575L^k0, v1=0x646f72616e646f6dL^k1, v2=0x6c7967656e657261L^k0, v3=0x7465646279746573L^k1;
        v3 ^= salt;
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        v0 ^= salt; v2 ^= 0xbbL;
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        long r0=v3,r1=v2,r2=v1,r3=v0;
        v0^=k0; v1^=k1; v2^=k0; v3^=k1;
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        long r4=v3,r5=v2,r6=v1,r7=v0;
        long r8=((r0^r4)&~7L)|3L;
        U.putLong(mem+16384L,r8);
        int bf=0; long sp=16384L;
        for (int rep=0; rep<4; rep++) {
            int bc=32;
            // program 0
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r7=Long.rotateRight(r7 * r8,40); bf=(int)r7;
                r6 = (r6 >>> 2) - r4;
                r5 = Long.rotateRight(r5,54) - r6;
                r3 = (r3 >>> 2) + r5;
                r2 = (r2 ^ 33L) * r3;
                r0 = (r0 ^ 9L) * r7;
                r1 = (r1 | 9L) * r2;
                r4 = Long.rotateRight(r4,60) + r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 1
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r4=Long.rotateRight(r4 * r8,20); bf=(int)r4;
                r5 = (r5 >>> 2) - r3;
                r3 = (r3 >>> 3) + r5;
                r7 = (r7 ^ 9L) * r2;
                r0 = (r0 ^ 1L) * r7;
                r2 = Long.rotateRight(r2,45) ^ r4;
                r1 = (r1 ^ 1L) * r6;
                r6 = Long.rotateRight(r6,51) - r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 2
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r0=Long.rotateRight(r0 * r8,10); bf=(int)r0;
                r2 = (r2 >> 1) + r7;
                r6 = (r6 ^ 33L) * r4;
                r5 = (r5 >>> 1) ^ r3;
                r1 = (r1 | 33L) * r0;
                r4 = (r4 ^ 9L) * r6;
                r3 = Long.rotateRight(r3,24) ^ r1;
                r7 = (r7 >> 2) - r5;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 3
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r5=Long.rotateRight(r5 * r8,24); bf=(int)r5;
                r1 = (r1 >> 3) + r0;
                r6 = (r6 >>> 1) ^ r7;
                r3 = (r3 >>> 3) + r4;
                r2 = Long.rotateRight(r2,37) ^ r5;
                r4 = (r4 ^ 33L) * r3;
                r7 = (r7 + 9L) * r2;
                r0 = (r0 + 1L) * r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 4
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r1=Long.rotateRight(r1 * r8,49); bf=(int)r1;
                r0 = (r0 >>> 2) ^ r1;
                r4 = (r4 >>> 1) ^ r6;
                r5 = Long.rotateRight(r5,10) + r7;
                r7 = (r7 >> 2) - r5;
                r6 = (r6 | 9L) * r3;
                r3 = (r3 | 1L) * r0;
                r2 = (r2 | 9L) * r4;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 5
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r3=Long.rotateRight(r3 * r8,63); bf=(int)r3;
                r5 = (r5 ^ 1L) * r0;
                r1 = (r1 >> 3) - r4;
                r4 = (r4 >>> 2) + r1;
                r2 = (r2 >>> 2) - r3;
                r7 = (r7 | 9L) * r6;
                r6 = (r6 ^ 33L) * r7;
                r0 = (r0 >>> 2) ^ r2;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 6
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r4=Long.rotateRight(r4 * r8,59); bf=(int)r4;
                r3 = (r3 | 1L) * r7;
                r6 = (r6 >>> 2) ^ r3;
                r0 = (r0 + 9L) * r6;
                r2 = Long.rotateRight(r2,11) ^ r0;
                r5 = (r5 >> 1) - r2;
                r7 = (r7 ^ 9L) * r1;
                r1 = Long.rotateRight(r1,5) - r5;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 7
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r0=Long.rotateRight(r0 * r8,57); bf=(int)r0;
                r4 = (r4 >> 3) + r7;
                r2 = (r2 >>> 2) - r4;
                r6 = (r6 + 1L) * r1;
                r3 = (r3 >>> 2) + r5;
                r5 = Long.rotateRight(r5,6) ^ r6;
                r7 = (r7 | 1L) * r3;
                r1 = (r1 ^ 1L) * r0;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 8
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r6=Long.rotateRight(r6 * r8,35); bf=(int)r6;
                r2 = Long.rotateRight(r2,18) + r1;
                r7 = (r7 + 1L) * r6;
                r3 = (r3 + 33L) * r5;
                r0 = (r0 >> 3) + r4;
                r1 = (r1 >>> 3) - r3;
                r5 = (r5 | 9L) * r0;
                r4 = (r4 >> 3) ^ r2;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 9
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r4=Long.rotateRight(r4 * r8,34); bf=(int)r4;
                r1 = Long.rotateRight(r1,16) + r3;
                r6 = (r6 >> 1) ^ r7;
                r7 = (r7 ^ 33L) * r1;
                r2 = (r2 ^ 33L) * r0;
                r0 = Long.rotateRight(r0,32) + r2;
                r3 = (r3 >> 1) + r6;
                r5 = (r5 ^ 9L) * r4;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 10
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r1=Long.rotateRight(r1 * r8,1); bf=(int)r1;
                r7 = (r7 >>> 1) ^ r6;
                r2 = (r2 ^ 1L) * r3;
                r3 = (r3 | 1L) * r7;
                r5 = (r5 ^ 33L) * r4;
                r4 = (r4 >>> 1) - r5;
                r0 = Long.rotateRight(r0,9) ^ r1;
                r6 = (r6 >> 2) ^ r2;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 11
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r3=Long.rotateRight(r3 * r8,17); bf=(int)r3;
                r0 = Long.rotateRight(r0,35) + r3;
                r5 = (r5 >>> 1) ^ r1;
                r4 = (r4 | 33L) * r7;
                r1 = Long.rotateRight(r1,7) - r4;
                r2 = (r2 | 9L) * r6;
                r7 = (r7 ^ 33L) * r5;
                r6 = Long.rotateRight(r6,55) ^ r0;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 12
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r0=Long.rotateRight(r0 * r8,21); bf=(int)r0;
                r5 = (r5 >> 3) ^ r6;
                r2 = (r2 >>> 2) + r5;
                r7 = (r7 | 9L) * r2;
                r3 = (r3 >> 1) ^ r7;
                r1 = (r1 | 1L) * r3;
                r6 = (r6 | 1L) * r0;
                r4 = (r4 >> 1) ^ r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 13
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r5=Long.rotateRight(r5 * r8,41); bf=(int)r5;
                r6 = Long.rotateRight(r6,55) + r3;
                r7 = (r7 >>> 1) ^ r6;
                r4 = (r4 ^ 9L) * r2;
                r1 = Long.rotateRight(r1,36) - r0;
                r2 = (r2 >>> 2) ^ r4;
                r3 = (r3 | 1L) * r5;
                r0 = (r0 ^ 33L) * r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 14
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r6=Long.rotateRight(r6 * r8,19); bf=(int)r6;
                r3 = Long.rotateRight(r3,61) + r6;
                r5 = (r5 + 1L) * r2;
                r7 = Long.rotateRight(r7,54) - r4;
                r2 = (r2 >>> 2) ^ r5;
                r4 = (r4 + 33L) * r7;
                r1 = (r1 | 1L) * r0;
                r0 = Long.rotateRight(r0,62) + r3;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 15
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r1=Long.rotateRight(r1 * r8,50); bf=(int)r1;
                r4 = Long.rotateRight(r4,35) + r6;
                r2 = (r2 | 1L) * r5;
                r3 = (r3 ^ 1L) * r1;
                r6 = (r6 >> 1) + r3;
                r5 = (r5 >> 2) - r2;
                r0 = (r0 | 33L) * r7;
                r7 = (r7 >> 2) + r0;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 16
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r0=Long.rotateRight(r0 * r8,31); bf=(int)r0;
                r5 = (r5 ^ 33L) * r6;
                r7 = (r7 >> 1) - r1;
                r3 = (r3 | 9L) * r0;
                r6 = Long.rotateRight(r6,36) + r2;
                r4 = Long.rotateRight(r4,37) ^ r5;
                r1 = (r1 | 1L) * r3;
                r2 = (r2 >>> 2) + r7;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 17
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r3=Long.rotateRight(r3 * r8,58); bf=(int)r3;
                r7 = (r7 >>> 3) - r6;
                r6 = (r6 ^ 33L) * r7;
                r5 = (r5 >>> 1) + r2;
                r4 = (r4 >>> 1) ^ r0;
                r0 = (r0 | 1L) * r5;
                r2 = (r2 >> 1) ^ r4;
                r1 = (r1 ^ 33L) * r3;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 18
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r5=Long.rotateRight(r5 * r8,34); bf=(int)r5;
                r7 = (r7 >> 2) - r6;
                r4 = Long.rotateRight(r4,1) - r7;
                r0 = (r0 >>> 2) + r4;
                r2 = (r2 ^ 9L) * r0;
                r1 = (r1 >>> 2) ^ r5;
                r3 = (r3 | 33L) * r2;
                r6 = (r6 ^ 33L) * r3;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 19
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r2=Long.rotateRight(r2 * r8,11); bf=(int)r2;
                r6 = Long.rotateRight(r6,55) ^ r7;
                r0 = (r0 >> 3) ^ r3;
                r7 = (r7 >>> 1) ^ r0;
                r4 = (r4 + 1L) * r2;
                r3 = (r3 >> 3) ^ r4;
                r1 = (r1 ^ 9L) * r5;
                r5 = (r5 ^ 33L) * r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 20
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r0=Long.rotateRight(r0 * r8,24); bf=(int)r0;
                r2 = (r2 >>> 1) + r4;
                r6 = (r6 + 1L) * r3;
                r1 = (r1 | 1L) * r7;
                r4 = (r4 >>> 1) + r2;
                r3 = (r3 | 9L) * r1;
                r5 = Long.rotateRight(r5,42) ^ r6;
                r7 = (r7 >> 3) ^ r0;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 21
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r1=Long.rotateRight(r1 * r8,20); bf=(int)r1;
                r7 = (r7 ^ 9L) * r2;
                r5 = Long.rotateRight(r5,44) - r6;
                r2 = (r2 >> 3) ^ r5;
                r4 = (r4 >> 2) ^ r1;
                r0 = (r0 >> 3) - r3;
                r3 = (r3 | 33L) * r0;
                r6 = (r6 + 9L) * r4;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 22
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r7=Long.rotateRight(r7 * r8,22); bf=(int)r7;
                r5 = (r5 >> 2) + r0;
                r6 = (r6 >>> 2) + r4;
                r2 = (r2 ^ 33L) * r6;
                r0 = (r0 | 9L) * r1;
                r3 = Long.rotateRight(r3,26) ^ r7;
                r4 = Long.rotateRight(r4,1) + r3;
                r1 = (r1 ^ 33L) * r5;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 23
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r7=Long.rotateRight(r7 * r8,59); bf=(int)r7;
                r0 = (r0 >> 2) + r1;
                r5 = (r5 >>> 1) - r2;
                r1 = (r1 ^ 1L) * r0;
                r2 = (r2 | 9L) * r3;
                r3 = (r3 + 33L) * r5;
                r6 = (r6 >> 2) ^ r7;
                r4 = (r4 >>> 2) + r6;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 24
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r7=Long.rotateRight(r7 * r8,21); bf=(int)r7;
                r6 = (r6 >> 3) ^ r3;
                r1 = (r1 ^ 33L) * r6;
                r5 = (r5 + 9L) * r1;
                r0 = (r0 >>> 3) + r7;
                r4 = Long.rotateRight(r4,31) - r5;
                r2 = (r2 ^ 9L) * r4;
                r3 = (r3 >>> 2) ^ r2;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 25
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r4=Long.rotateRight(r4 * r8,33); bf=(int)r4;
                r3 = (r3 >>> 2) + r7;
                r0 = (r0 >>> 3) + r3;
                r1 = (r1 ^ 1L) * r5;
                r6 = (r6 >> 3) - r4;
                r7 = (r7 | 1L) * r2;
                r2 = (r2 ^ 9L) * r1;
                r5 = (r5 >>> 2) + r6;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 26
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r3=Long.rotateRight(r3 * r8,4); bf=(int)r3;
                r4 = (r4 >>> 2) + r5;
                r5 = (r5 >>> 3) + r4;
                r7 = (r7 ^ 1L) * r3;
                r2 = (r2 ^ 9L) * r1;
                r1 = (r1 | 9L) * r2;
                r0 = (r0 >>> 2) + r6;
                r6 = Long.rotateRight(r6,22) ^ r0;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 27
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r4=Long.rotateRight(r4 * r8,3); bf=(int)r4;
                r6 = (r6 >>> 1) - r1;
                r1 = (r1 | 1L) * r4;
                r0 = (r0 | 33L) * r5;
                r3 = Long.rotateRight(r3,19) - r0;
                r7 = (r7 >>> 3) - r2;
                r5 = (r5 >> 1) - r6;
                r2 = (r2 | 9L) * r7;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 28
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r6=Long.rotateRight(r6 * r8,3); bf=(int)r6;
                r0 = (r0 | 9L) * r3;
                r5 = (r5 + 33L) * r7;
                r3 = (r3 | 9L) * r2;
                r2 = (r2 >>> 3) - r0;
                r1 = (r1 >>> 3) + r5;
                r4 = (r4 >> 2) - r6;
                r7 = (r7 >> 3) - r4;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 29
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r0=Long.rotateRight(r0 * r8,29); bf=(int)r0;
                r1 = (r1 | 9L) * r0;
                r4 = (r4 ^ 9L) * r5;
                r5 = Long.rotateRight(r5,47) ^ r4;
                r3 = Long.rotateRight(r3,13) + r2;
                r6 = Long.rotateRight(r6,51) - r3;
                r2 = (r2 ^ 33L) * r7;
                r7 = Long.rotateRight(r7,31) ^ r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 30
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r6=Long.rotateRight(r6 * r8,52); bf=(int)r6;
                r3 = Long.rotateRight(r3,39) ^ r7;
                r4 = Long.rotateRight(r4,56) - r3;
                r5 = (r5 >> 3) - r4;
                r2 = (r2 | 1L) * r5;
                r0 = (r0 >>> 3) - r6;
                r1 = (r1 + 33L) * r2;
                r7 = (r7 | 1L) * r1;
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 31
            do {
                sp -= 64;
                U.putLong(mem + sp + 56L, r0);
                U.putLong(mem + sp + 48L, r1);
                U.putLong(mem + sp + 40L, r2);
                U.putLong(mem + sp + 32L, r3);
                U.putLong(mem + sp + 24L, r4);
                U.putLong(mem + sp + 16L, r5);
                U.putLong(mem + sp + 8L, r6);
                U.putLong(mem + sp + 0L, r7);
                r7=Long.rotateRight(r7 * r8,5); bf=(int)r7;
                r6 = (r6 >>> 2) ^ r1;
                r4 = (r4 | 9L) * r5;
                r2 = (r2 | 1L) * r0;
                r5 = (r5 >> 3) ^ r2;
                r0 = Long.rotateRight(r0,62) ^ r4;
                r1 = (r1 + 1L) * r6;
                r3 = (r3 >> 2) - r7;
                if (bc==0) break;
                bc--;
            } while (true);
        }
        for (int rep=0; rep<4; rep++) {
            int bc=32;
            // program 0
            do {
                r7=Long.rotateRight(r7 * r8,40); bf=(int)r7;
                r6 = (r6 >>> 2) - U.getLong(mem + (r4 & 16383L));
                r5 = Long.rotateRight(r5,54) - U.getLong(mem + (r6 & 16383L));
                r3 = (r3 >>> 2) + U.getLong(mem + (r5 & 16383L));
                r2 = (r2 ^ 33L) * U.getLong(mem + (r3 & 16383L));
                r0 = (r0 ^ 9L) * U.getLong(mem + (r7 & 16383L));
                r1 = (r1 | 9L) * U.getLong(mem + (r2 & 16383L));
                r4 = Long.rotateRight(r4,60) + U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 1
            do {
                r4=Long.rotateRight(r4 * r8,20); bf=(int)r4;
                r5 = (r5 >>> 2) - U.getLong(mem + (r3 & 16383L));
                r3 = (r3 >>> 3) + U.getLong(mem + (r5 & 16383L));
                r7 = (r7 ^ 9L) * U.getLong(mem + (r2 & 16383L));
                r0 = (r0 ^ 1L) * U.getLong(mem + (r7 & 16383L));
                r2 = Long.rotateRight(r2,45) ^ U.getLong(mem + (r4 & 16383L));
                r1 = (r1 ^ 1L) * U.getLong(mem + (r6 & 16383L));
                r6 = Long.rotateRight(r6,51) - U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 2
            do {
                r0=Long.rotateRight(r0 * r8,10); bf=(int)r0;
                r2 = (r2 >> 1) + U.getLong(mem + (r7 & 16383L));
                r6 = (r6 ^ 33L) * U.getLong(mem + (r4 & 16383L));
                r5 = (r5 >>> 1) ^ U.getLong(mem + (r3 & 16383L));
                r1 = (r1 | 33L) * U.getLong(mem + (r0 & 16383L));
                r4 = (r4 ^ 9L) * U.getLong(mem + (r6 & 16383L));
                r3 = Long.rotateRight(r3,24) ^ U.getLong(mem + (r1 & 16383L));
                r7 = (r7 >> 2) - U.getLong(mem + (r5 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 3
            do {
                r5=Long.rotateRight(r5 * r8,24); bf=(int)r5;
                r1 = (r1 >> 3) + U.getLong(mem + (r0 & 16383L));
                r6 = (r6 >>> 1) ^ U.getLong(mem + (r7 & 16383L));
                r3 = (r3 >>> 3) + U.getLong(mem + (r4 & 16383L));
                r2 = Long.rotateRight(r2,37) ^ U.getLong(mem + (r5 & 16383L));
                r4 = (r4 ^ 33L) * U.getLong(mem + (r3 & 16383L));
                r7 = (r7 + 9L) * U.getLong(mem + (r2 & 16383L));
                r0 = (r0 + 1L) * U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 4
            do {
                r1=Long.rotateRight(r1 * r8,49); bf=(int)r1;
                r0 = (r0 >>> 2) ^ U.getLong(mem + (r1 & 16383L));
                r4 = (r4 >>> 1) ^ U.getLong(mem + (r6 & 16383L));
                r5 = Long.rotateRight(r5,10) + U.getLong(mem + (r7 & 16383L));
                r7 = (r7 >> 2) - U.getLong(mem + (r5 & 16383L));
                r6 = (r6 | 9L) * U.getLong(mem + (r3 & 16383L));
                r3 = (r3 | 1L) * U.getLong(mem + (r0 & 16383L));
                r2 = (r2 | 9L) * U.getLong(mem + (r4 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 5
            do {
                r3=Long.rotateRight(r3 * r8,63); bf=(int)r3;
                r5 = (r5 ^ 1L) * U.getLong(mem + (r0 & 16383L));
                r1 = (r1 >> 3) - U.getLong(mem + (r4 & 16383L));
                r4 = (r4 >>> 2) + U.getLong(mem + (r1 & 16383L));
                r2 = (r2 >>> 2) - U.getLong(mem + (r3 & 16383L));
                r7 = (r7 | 9L) * U.getLong(mem + (r6 & 16383L));
                r6 = (r6 ^ 33L) * U.getLong(mem + (r7 & 16383L));
                r0 = (r0 >>> 2) ^ U.getLong(mem + (r2 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 6
            do {
                r4=Long.rotateRight(r4 * r8,59); bf=(int)r4;
                r3 = (r3 | 1L) * U.getLong(mem + (r7 & 16383L));
                r6 = (r6 >>> 2) ^ U.getLong(mem + (r3 & 16383L));
                r0 = (r0 + 9L) * U.getLong(mem + (r6 & 16383L));
                r2 = Long.rotateRight(r2,11) ^ U.getLong(mem + (r0 & 16383L));
                r5 = (r5 >> 1) - U.getLong(mem + (r2 & 16383L));
                r7 = (r7 ^ 9L) * U.getLong(mem + (r1 & 16383L));
                r1 = Long.rotateRight(r1,5) - U.getLong(mem + (r5 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 7
            do {
                r0=Long.rotateRight(r0 * r8,57); bf=(int)r0;
                r4 = (r4 >> 3) + U.getLong(mem + (r7 & 16383L));
                r2 = (r2 >>> 2) - U.getLong(mem + (r4 & 16383L));
                r6 = (r6 + 1L) * U.getLong(mem + (r1 & 16383L));
                r3 = (r3 >>> 2) + U.getLong(mem + (r5 & 16383L));
                r5 = Long.rotateRight(r5,6) ^ U.getLong(mem + (r6 & 16383L));
                r7 = (r7 | 1L) * U.getLong(mem + (r3 & 16383L));
                r1 = (r1 ^ 1L) * U.getLong(mem + (r0 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 8
            do {
                r6=Long.rotateRight(r6 * r8,35); bf=(int)r6;
                r2 = Long.rotateRight(r2,18) + U.getLong(mem + (r1 & 16383L));
                r7 = (r7 + 1L) * U.getLong(mem + (r6 & 16383L));
                r3 = (r3 + 33L) * U.getLong(mem + (r5 & 16383L));
                r0 = (r0 >> 3) + U.getLong(mem + (r4 & 16383L));
                r1 = (r1 >>> 3) - U.getLong(mem + (r3 & 16383L));
                r5 = (r5 | 9L) * U.getLong(mem + (r0 & 16383L));
                r4 = (r4 >> 3) ^ U.getLong(mem + (r2 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 9
            do {
                r4=Long.rotateRight(r4 * r8,34); bf=(int)r4;
                r1 = Long.rotateRight(r1,16) + U.getLong(mem + (r3 & 16383L));
                r6 = (r6 >> 1) ^ U.getLong(mem + (r7 & 16383L));
                r7 = (r7 ^ 33L) * U.getLong(mem + (r1 & 16383L));
                r2 = (r2 ^ 33L) * U.getLong(mem + (r0 & 16383L));
                r0 = Long.rotateRight(r0,32) + U.getLong(mem + (r2 & 16383L));
                r3 = (r3 >> 1) + U.getLong(mem + (r6 & 16383L));
                r5 = (r5 ^ 9L) * U.getLong(mem + (r4 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 10
            do {
                r1=Long.rotateRight(r1 * r8,1); bf=(int)r1;
                r7 = (r7 >>> 1) ^ U.getLong(mem + (r6 & 16383L));
                r2 = (r2 ^ 1L) * U.getLong(mem + (r3 & 16383L));
                r3 = (r3 | 1L) * U.getLong(mem + (r7 & 16383L));
                r5 = (r5 ^ 33L) * U.getLong(mem + (r4 & 16383L));
                r4 = (r4 >>> 1) - U.getLong(mem + (r5 & 16383L));
                r0 = Long.rotateRight(r0,9) ^ U.getLong(mem + (r1 & 16383L));
                r6 = (r6 >> 2) ^ U.getLong(mem + (r2 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 11
            do {
                r3=Long.rotateRight(r3 * r8,17); bf=(int)r3;
                r0 = Long.rotateRight(r0,35) + U.getLong(mem + (r3 & 16383L));
                r5 = (r5 >>> 1) ^ U.getLong(mem + (r1 & 16383L));
                r4 = (r4 | 33L) * U.getLong(mem + (r7 & 16383L));
                r1 = Long.rotateRight(r1,7) - U.getLong(mem + (r4 & 16383L));
                r2 = (r2 | 9L) * U.getLong(mem + (r6 & 16383L));
                r7 = (r7 ^ 33L) * U.getLong(mem + (r5 & 16383L));
                r6 = Long.rotateRight(r6,55) ^ U.getLong(mem + (r0 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 12
            do {
                r0=Long.rotateRight(r0 * r8,21); bf=(int)r0;
                r5 = (r5 >> 3) ^ U.getLong(mem + (r6 & 16383L));
                r2 = (r2 >>> 2) + U.getLong(mem + (r5 & 16383L));
                r7 = (r7 | 9L) * U.getLong(mem + (r2 & 16383L));
                r3 = (r3 >> 1) ^ U.getLong(mem + (r7 & 16383L));
                r1 = (r1 | 1L) * U.getLong(mem + (r3 & 16383L));
                r6 = (r6 | 1L) * U.getLong(mem + (r0 & 16383L));
                r4 = (r4 >> 1) ^ U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 13
            do {
                r5=Long.rotateRight(r5 * r8,41); bf=(int)r5;
                r6 = Long.rotateRight(r6,55) + U.getLong(mem + (r3 & 16383L));
                r7 = (r7 >>> 1) ^ U.getLong(mem + (r6 & 16383L));
                r4 = (r4 ^ 9L) * U.getLong(mem + (r2 & 16383L));
                r1 = Long.rotateRight(r1,36) - U.getLong(mem + (r0 & 16383L));
                r2 = (r2 >>> 2) ^ U.getLong(mem + (r4 & 16383L));
                r3 = (r3 | 1L) * U.getLong(mem + (r5 & 16383L));
                r0 = (r0 ^ 33L) * U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 14
            do {
                r6=Long.rotateRight(r6 * r8,19); bf=(int)r6;
                r3 = Long.rotateRight(r3,61) + U.getLong(mem + (r6 & 16383L));
                r5 = (r5 + 1L) * U.getLong(mem + (r2 & 16383L));
                r7 = Long.rotateRight(r7,54) - U.getLong(mem + (r4 & 16383L));
                r2 = (r2 >>> 2) ^ U.getLong(mem + (r5 & 16383L));
                r4 = (r4 + 33L) * U.getLong(mem + (r7 & 16383L));
                r1 = (r1 | 1L) * U.getLong(mem + (r0 & 16383L));
                r0 = Long.rotateRight(r0,62) + U.getLong(mem + (r3 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 15
            do {
                r1=Long.rotateRight(r1 * r8,50); bf=(int)r1;
                r4 = Long.rotateRight(r4,35) + U.getLong(mem + (r6 & 16383L));
                r2 = (r2 | 1L) * U.getLong(mem + (r5 & 16383L));
                r3 = (r3 ^ 1L) * U.getLong(mem + (r1 & 16383L));
                r6 = (r6 >> 1) + U.getLong(mem + (r3 & 16383L));
                r5 = (r5 >> 2) - U.getLong(mem + (r2 & 16383L));
                r0 = (r0 | 33L) * U.getLong(mem + (r7 & 16383L));
                r7 = (r7 >> 2) + U.getLong(mem + (r0 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 16
            do {
                r0=Long.rotateRight(r0 * r8,31); bf=(int)r0;
                r5 = (r5 ^ 33L) * U.getLong(mem + (r6 & 16383L));
                r7 = (r7 >> 1) - U.getLong(mem + (r1 & 16383L));
                r3 = (r3 | 9L) * U.getLong(mem + (r0 & 16383L));
                r6 = Long.rotateRight(r6,36) + U.getLong(mem + (r2 & 16383L));
                r4 = Long.rotateRight(r4,37) ^ U.getLong(mem + (r5 & 16383L));
                r1 = (r1 | 1L) * U.getLong(mem + (r3 & 16383L));
                r2 = (r2 >>> 2) + U.getLong(mem + (r7 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 17
            do {
                r3=Long.rotateRight(r3 * r8,58); bf=(int)r3;
                r7 = (r7 >>> 3) - U.getLong(mem + (r6 & 16383L));
                r6 = (r6 ^ 33L) * U.getLong(mem + (r7 & 16383L));
                r5 = (r5 >>> 1) + U.getLong(mem + (r2 & 16383L));
                r4 = (r4 >>> 1) ^ U.getLong(mem + (r0 & 16383L));
                r0 = (r0 | 1L) * U.getLong(mem + (r5 & 16383L));
                r2 = (r2 >> 1) ^ U.getLong(mem + (r4 & 16383L));
                r1 = (r1 ^ 33L) * U.getLong(mem + (r3 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 18
            do {
                r5=Long.rotateRight(r5 * r8,34); bf=(int)r5;
                r7 = (r7 >> 2) - U.getLong(mem + (r6 & 16383L));
                r4 = Long.rotateRight(r4,1) - U.getLong(mem + (r7 & 16383L));
                r0 = (r0 >>> 2) + U.getLong(mem + (r4 & 16383L));
                r2 = (r2 ^ 9L) * U.getLong(mem + (r0 & 16383L));
                r1 = (r1 >>> 2) ^ U.getLong(mem + (r5 & 16383L));
                r3 = (r3 | 33L) * U.getLong(mem + (r2 & 16383L));
                r6 = (r6 ^ 33L) * U.getLong(mem + (r3 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 19
            do {
                r2=Long.rotateRight(r2 * r8,11); bf=(int)r2;
                r6 = Long.rotateRight(r6,55) ^ U.getLong(mem + (r7 & 16383L));
                r0 = (r0 >> 3) ^ U.getLong(mem + (r3 & 16383L));
                r7 = (r7 >>> 1) ^ U.getLong(mem + (r0 & 16383L));
                r4 = (r4 + 1L) * U.getLong(mem + (r2 & 16383L));
                r3 = (r3 >> 3) ^ U.getLong(mem + (r4 & 16383L));
                r1 = (r1 ^ 9L) * U.getLong(mem + (r5 & 16383L));
                r5 = (r5 ^ 33L) * U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 20
            do {
                r0=Long.rotateRight(r0 * r8,24); bf=(int)r0;
                r2 = (r2 >>> 1) + U.getLong(mem + (r4 & 16383L));
                r6 = (r6 + 1L) * U.getLong(mem + (r3 & 16383L));
                r1 = (r1 | 1L) * U.getLong(mem + (r7 & 16383L));
                r4 = (r4 >>> 1) + U.getLong(mem + (r2 & 16383L));
                r3 = (r3 | 9L) * U.getLong(mem + (r1 & 16383L));
                r5 = Long.rotateRight(r5,42) ^ U.getLong(mem + (r6 & 16383L));
                r7 = (r7 >> 3) ^ U.getLong(mem + (r0 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 21
            do {
                r1=Long.rotateRight(r1 * r8,20); bf=(int)r1;
                r7 = (r7 ^ 9L) * U.getLong(mem + (r2 & 16383L));
                r5 = Long.rotateRight(r5,44) - U.getLong(mem + (r6 & 16383L));
                r2 = (r2 >> 3) ^ U.getLong(mem + (r5 & 16383L));
                r4 = (r4 >> 2) ^ U.getLong(mem + (r1 & 16383L));
                r0 = (r0 >> 3) - U.getLong(mem + (r3 & 16383L));
                r3 = (r3 | 33L) * U.getLong(mem + (r0 & 16383L));
                r6 = (r6 + 9L) * U.getLong(mem + (r4 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 22
            do {
                r7=Long.rotateRight(r7 * r8,22); bf=(int)r7;
                r5 = (r5 >> 2) + U.getLong(mem + (r0 & 16383L));
                r6 = (r6 >>> 2) + U.getLong(mem + (r4 & 16383L));
                r2 = (r2 ^ 33L) * U.getLong(mem + (r6 & 16383L));
                r0 = (r0 | 9L) * U.getLong(mem + (r1 & 16383L));
                r3 = Long.rotateRight(r3,26) ^ U.getLong(mem + (r7 & 16383L));
                r4 = Long.rotateRight(r4,1) + U.getLong(mem + (r3 & 16383L));
                r1 = (r1 ^ 33L) * U.getLong(mem + (r5 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 23
            do {
                r7=Long.rotateRight(r7 * r8,59); bf=(int)r7;
                r0 = (r0 >> 2) + U.getLong(mem + (r1 & 16383L));
                r5 = (r5 >>> 1) - U.getLong(mem + (r2 & 16383L));
                r1 = (r1 ^ 1L) * U.getLong(mem + (r0 & 16383L));
                r2 = (r2 | 9L) * U.getLong(mem + (r3 & 16383L));
                r3 = (r3 + 33L) * U.getLong(mem + (r5 & 16383L));
                r6 = (r6 >> 2) ^ U.getLong(mem + (r7 & 16383L));
                r4 = (r4 >>> 2) + U.getLong(mem + (r6 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 24
            do {
                r7=Long.rotateRight(r7 * r8,21); bf=(int)r7;
                r6 = (r6 >> 3) ^ U.getLong(mem + (r3 & 16383L));
                r1 = (r1 ^ 33L) * U.getLong(mem + (r6 & 16383L));
                r5 = (r5 + 9L) * U.getLong(mem + (r1 & 16383L));
                r0 = (r0 >>> 3) + U.getLong(mem + (r7 & 16383L));
                r4 = Long.rotateRight(r4,31) - U.getLong(mem + (r5 & 16383L));
                r2 = (r2 ^ 9L) * U.getLong(mem + (r4 & 16383L));
                r3 = (r3 >>> 2) ^ U.getLong(mem + (r2 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 25
            do {
                r4=Long.rotateRight(r4 * r8,33); bf=(int)r4;
                r3 = (r3 >>> 2) + U.getLong(mem + (r7 & 16383L));
                r0 = (r0 >>> 3) + U.getLong(mem + (r3 & 16383L));
                r1 = (r1 ^ 1L) * U.getLong(mem + (r5 & 16383L));
                r6 = (r6 >> 3) - U.getLong(mem + (r4 & 16383L));
                r7 = (r7 | 1L) * U.getLong(mem + (r2 & 16383L));
                r2 = (r2 ^ 9L) * U.getLong(mem + (r1 & 16383L));
                r5 = (r5 >>> 2) + U.getLong(mem + (r6 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 26
            do {
                r3=Long.rotateRight(r3 * r8,4); bf=(int)r3;
                r4 = (r4 >>> 2) + U.getLong(mem + (r5 & 16383L));
                r5 = (r5 >>> 3) + U.getLong(mem + (r4 & 16383L));
                r7 = (r7 ^ 1L) * U.getLong(mem + (r3 & 16383L));
                r2 = (r2 ^ 9L) * U.getLong(mem + (r1 & 16383L));
                r1 = (r1 | 9L) * U.getLong(mem + (r2 & 16383L));
                r0 = (r0 >>> 2) + U.getLong(mem + (r6 & 16383L));
                r6 = Long.rotateRight(r6,22) ^ U.getLong(mem + (r0 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 27
            do {
                r4=Long.rotateRight(r4 * r8,3); bf=(int)r4;
                r6 = (r6 >>> 1) - U.getLong(mem + (r1 & 16383L));
                r1 = (r1 | 1L) * U.getLong(mem + (r4 & 16383L));
                r0 = (r0 | 33L) * U.getLong(mem + (r5 & 16383L));
                r3 = Long.rotateRight(r3,19) - U.getLong(mem + (r0 & 16383L));
                r7 = (r7 >>> 3) - U.getLong(mem + (r2 & 16383L));
                r5 = (r5 >> 1) - U.getLong(mem + (r6 & 16383L));
                r2 = (r2 | 9L) * U.getLong(mem + (r7 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 28
            do {
                r6=Long.rotateRight(r6 * r8,3); bf=(int)r6;
                r0 = (r0 | 9L) * U.getLong(mem + (r3 & 16383L));
                r5 = (r5 + 33L) * U.getLong(mem + (r7 & 16383L));
                r3 = (r3 | 9L) * U.getLong(mem + (r2 & 16383L));
                r2 = (r2 >>> 3) - U.getLong(mem + (r0 & 16383L));
                r1 = (r1 >>> 3) + U.getLong(mem + (r5 & 16383L));
                r4 = (r4 >> 2) - U.getLong(mem + (r6 & 16383L));
                r7 = (r7 >> 3) - U.getLong(mem + (r4 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 29
            do {
                r0=Long.rotateRight(r0 * r8,29); bf=(int)r0;
                r1 = (r1 | 9L) * U.getLong(mem + (r0 & 16383L));
                r4 = (r4 ^ 9L) * U.getLong(mem + (r5 & 16383L));
                r5 = Long.rotateRight(r5,47) ^ U.getLong(mem + (r4 & 16383L));
                r3 = Long.rotateRight(r3,13) + U.getLong(mem + (r2 & 16383L));
                r6 = Long.rotateRight(r6,51) - U.getLong(mem + (r3 & 16383L));
                r2 = (r2 ^ 33L) * U.getLong(mem + (r7 & 16383L));
                r7 = Long.rotateRight(r7,31) ^ U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 30
            do {
                r6=Long.rotateRight(r6 * r8,52); bf=(int)r6;
                r3 = Long.rotateRight(r3,39) ^ U.getLong(mem + (r7 & 16383L));
                r4 = Long.rotateRight(r4,56) - U.getLong(mem + (r3 & 16383L));
                r5 = (r5 >> 3) - U.getLong(mem + (r4 & 16383L));
                r2 = (r2 | 1L) * U.getLong(mem + (r5 & 16383L));
                r0 = (r0 >>> 3) - U.getLong(mem + (r6 & 16383L));
                r1 = (r1 + 33L) * U.getLong(mem + (r2 & 16383L));
                r7 = (r7 | 1L) * U.getLong(mem + (r1 & 16383L));
                if (bc==0) break;
                if ((bf & 32) != 0) break;
                bc--;
            } while (true);
            // program 31
            do {
                r7=Long.rotateRight(r7 * r8,5); bf=(int)r7;
                r6 = (r6 >>> 2) ^ U.getLong(mem + (r1 & 16383L));
                r4 = (r4 | 9L) * U.getLong(mem + (r5 & 16383L));
                r2 = (r2 | 1L) * U.getLong(mem + (r0 & 16383L));
                r5 = (r5 >> 3) ^ U.getLong(mem + (r2 & 16383L));
                r0 = Long.rotateRight(r0,62) ^ U.getLong(mem + (r4 & 16383L));
                r1 = (r1 + 1L) * U.getLong(mem + (r6 & 16383L));
                r3 = (r3 >> 2) - U.getLong(mem + (r7 & 16383L));
                if (bc==0) break;
                bc--;
            } while (true);
        }
        v0=r0;v1=r1;v2=r2;v3=r3;
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        long out3=v3;
        v0=r4;v1=r5;v2=r6;v3=r7;
        v0 += v1; v2 += v3;
        v1 = Long.rotateLeft(v1,13); v3 = Long.rotateLeft(v3,16);
        v1 ^= v0; v3 ^= v2; v0 = Long.rotateLeft(v0,32);
        v2 += v1; v0 += v3;
        v1 = Long.rotateLeft(v1,17); v3 = Long.rotateLeft(v3,21);
        v1 ^= v2; v3 ^= v0; v2 = Long.rotateLeft(v2,32);
        return out3 ^ v3 ^ r8;
    }
    public static void main(String[] args) {
        long mem=U.allocateMemory(16392); try {
            long c=hash(123456L,mem); System.out.printf("CHECK %016x%n",c);
            for(int i=0;i<30000;i++) hash(i,mem);
            final int N=300000;
            for(int s=0;s<7;s++){ long z=0; long t=System.nanoTime(); for(int i=0;i<N;i++) z^=hash(i,mem); long dt=System.nanoTime()-t; System.out.printf("RATE %.3f sink=%016x%n",N/(dt*1e-9),z); }
        } finally { U.freeMemory(mem); }
    }
}