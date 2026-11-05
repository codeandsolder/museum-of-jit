//! Reference adapter: production challenge-solvers SIMD SHA-256 nonce engine.
//! The actual solver, not a synthetic Rust implementation. Git revision is pinned.
use challenge_solvers::message::SingleBlockMessage;
use challenge_solvers::solver::{SOLVE_TYPE_MASK, Solver};
use std::time::Instant;

fn prefix() -> [u8; 42] {
    let mut buf = [b'x'; 42];
    buf[.."challenge-disasterer|sha256|".len()]
        .copy_from_slice(b"challenge-disasterer|sha256|");
    buf
}

fn run_backend<S>(message: &SingleBlockMessage, limit: u64) -> Result<u64, String>
where
    S: From<SingleBlockMessage> + Solver<Output = [u32; 8]>,
{
    let mut solver = S::from(message.clone());
    solver.set_limit(limit);
    // Full 64-bit equality test. A hit is exceedingly unlikely; fail rather than
    // silently misreport attempted work if one occurs.
    let solution = solver.solve_nonce_only::<SOLVE_TYPE_MASK>(
        0xbadd_cafe_1234_5678,
        u64::MAX,
    );
    if solution.is_some() {
        return Err("unexpected target collision during benchmark".into());
    }
    let attempts = solver.attempted_nonces();
    if attempts != limit {
        return Err(format!("attempt accounting mismatch: {attempts} != {limit}"));
    }
    Ok(attempts)
}

fn run(message: &SingleBlockMessage, count: u64) -> Result<u64, String> {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx512f") {
            return run_backend::<challenge_solvers::solver::x86::Avx512SingleBlockSolver>(
                message, count,
            );
        }
        if std::arch::is_x86_feature_detected!("avx2") {
            return run_backend::<challenge_solvers::solver::x86::Avx2SingleBlockSolver>(
                message, count,
            );
        }
    }
    run_backend::<challenge_solvers::solver::safe::SingleBlockSolver>(message, count)
}

fn check(message: &SingleBlockMessage) -> Result<String, String> {
    let mut solver = challenge_solvers::solver::safe::SingleBlockSolver::from(message.clone());
    solver.set_limit(1);
    let (nonce, digest) = solver
        .solve::<SOLVE_TYPE_MASK>(0, 0)
        .ok_or("reference oracle did not match first candidate")?;
    if nonce != 100_000_000 {
        return Err(format!("wrong first candidate: {nonce}"));
    }
    Ok(format!("{:08x}{:08x}", digest[0], digest[1]))
}

fn parse_env(name: &str, default: u64, minimum: u64) -> Result<u64, String> {
    let raw = std::env::var(name).unwrap_or_else(|_| default.to_string());
    let value: u64 = raw.parse().map_err(|_| format!("invalid {name}: {raw}"))?;
    if value < minimum || value > 160_000_000 {
        return Err(format!("{name} out of range: {value}"));
    }
    Ok(value)
}

fn main() -> Result<(), String> {
    let prefix = prefix();
    let message = SingleBlockMessage::new(&prefix, 0)
        .ok_or("challenge-solvers rejected 42-byte fixed prefix")?;
    if message.digit_index != 42 || message.nonce_addend != 0 || message.no_trailing_zeros {
        return Err("challenge-solvers changed fixture layout".into());
    }
    let expected = "b4b920f70c0de8df";
    let oracle = check(&message)?;
    if oracle != expected {
        return Err(format!("reference changed: expected {expected}, got {oracle}"));
    }

    let warmup = parse_env("MUSEUM_WARMUP", 30_000, 0)?;
    let iterations = parse_env("MUSEUM_ITERATIONS", 300_000, 1)?;
    let samples = parse_env("MUSEUM_SAMPLES", 7, 1)?;
    #[cfg(target_arch = "x86_64")]
    let lane_count = if std::arch::is_x86_feature_detected!("avx512f") { 16 }
        else if std::arch::is_x86_feature_detected!("avx2") { 8 }
        else { 1 };
    #[cfg(not(target_arch = "x86_64"))]
    let lane_count = 1;
    println!("ENGINE rust-challenge MODE simd VECTOR_LANES {lane_count}");
    println!("CHECK {oracle}");
    let _ = run(&message, warmup)?;
    for _ in 0..samples {
        let start = Instant::now();
        let work = run(&message, iterations)?;
        let seconds = start.elapsed().as_secs_f64();
        let rate = (work as f64) / seconds;
        // Provenance sink is exact attempted count, not digest XOR; the optimized
        // production solver only materializes matching digests.
        println!("RATE {rate:.6} sink={work:016x}");
    }
    Ok(())
}
