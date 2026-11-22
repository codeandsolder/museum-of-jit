// Low word first. Every operation is modulo 2^64 unless specified.
alias Word = vec2<u32>;
fn add64(a: Word, b: Word) -> Word {
    let lo = a.x + b.x;
    return Word(lo, a.y + b.y + u32(lo < a.x));
}
fn sub64(a: Word, b: Word) -> Word {
    return Word(a.x - b.x, a.y - b.y - u32(a.x < b.x));
}
fn mul32(a: u32, b: u32) -> Word {
    let al = a & 65535u;
    let bl = b & 65535u;
    let ah = a >> 16u;
    let bh = b >> 16u;
    let p = al * bl;
    let q = ah * bl;
    let r = al * bh;
    let t = (p >> 16u) + (q & 65535u) + (r & 65535u);
    return Word((p & 65535u) | (t << 16u), ah * bh + (q >> 16u) + (r >> 16u) + (t >> 16u));
}
fn mul64(a: Word, b: Word) -> Word {
    let p = mul32(a.x, b.x);
    return Word(p.x, p.y + a.x * b.y + a.y * b.x);
}
fn umulh(a: Word, b: Word) -> Word {
    let p = mul32(a.x, b.x);
    let q = mul32(a.x, b.y);
    let r = mul32(a.y, b.x);
    let s = mul32(a.y, b.y);
    let middle = add64(add64(Word(p.y, 0u), Word(q.x, 0u)), Word(r.x, 0u));
    return add64(add64(add64(s, Word(q.y, 0u)), Word(r.y, 0u)), Word(middle.y, 0u));
}
fn smulh(a: Word, b: Word) -> Word {
    var hi = umulh(a, b);
    if ((a.y & 0x80000000u) != 0u) { hi = sub64(hi, b); }
    if ((b.y & 0x80000000u) != 0u) { hi = sub64(hi, a); }
    return hi;
}
fn shl64(a: Word, n: u32) -> Word {
    if (n == 0u) { return a; }
    if (n >= 32u) { return Word(0u, a.x << (n - 32u)); }
    return Word(a.x << n, (a.y << n) | (a.x >> (32u - n)));
}
fn shr64(a: Word, n: u32) -> Word {
    if (n == 0u) { return a; }
    if (n >= 32u) { return Word(a.y >> (n - 32u), 0u); }
    return Word((a.x >> n) | (a.y << (32u - n)), a.y >> n);
}
fn asr64(a: Word, n: u32) -> Word {
    if (n == 0u) { return a; }
    let hi = bitcast<i32>(a.y);
    if (n >= 32u) { return Word(bitcast<u32>(hi >> (n - 32u)), bitcast<u32>(hi >> 31u)); }
    return Word((a.x >> n) | (a.y << (32u - n)), bitcast<u32>(hi >> n));
}
fn ror64(a: Word, n: u32) -> Word {
    if (n == 0u) { return a; }
    return shr64(a, n) | shl64(a, 64u - n);
}
alias State = array<Word, 4>;
fn sip(v: State) -> State {
    var a = add64(v[0], v[1]);
    var c = add64(v[2], v[3]);
    var b = ror64(v[1], 51u) ^ a;
    var d = ror64(v[3], 48u) ^ c;
    a = ror64(a, 32u);
    c = add64(c, b);
    a = add64(a, d);
    b = ror64(b, 47u) ^ c;
    d = ror64(d, 43u) ^ a;
    return State(a, b, ror64(c, 32u), d);
}
