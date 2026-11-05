//! Fixed-work SHA-NI versus portable-SIMD thread scaling on Linux/x86-64.
use challenge_solvers::message::SingleBlockMessage;
use challenge_solvers::solver::{SOLVE_TYPE_MASK, Solver};
use sha2::{Digest as _, Sha256};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

const TARGET: u64 = 0xbadd_cafe_1234_5678;
const SHA256_FIRST: &str = "b4b920f70c0de8df";

#[derive(Clone, Copy)]
enum Mode {
    ShaNi,
    Simd,
}
impl Mode {
    const fn label(self) -> &'static str {
        match self {
            Self::ShaNi => "sha-ni",
            Self::Simd => "portable-simd",
        }
    }
}

fn prefix(worker: usize) -> [u8; 42] {
    let mut result = [b'x'; 42];
    result[.."challenge-disasterer|sha256|".len()].copy_from_slice(b"challenge-disasterer|sha256|");
    // Worker zero retains the museum fixture. Others hash distinct messages
    // of exactly the same layout and length, identically for both backends.
    if worker != 0 {
        result[40] = b'0' + u8::try_from(worker).expect("worker ID fits");
    }
    result
}

fn run_solver<S>(
    msg: &SingleBlockMessage,
    limit: u64,
    target: u64,
    mask: u64,
) -> Result<Option<(u64, [u32; 8])>, String>
where
    S: From<SingleBlockMessage> + Solver<Output = [u32; 8]>,
{
    let mut solver = S::from(msg.clone());
    solver.set_limit(limit);
    let solution = solver.solve::<SOLVE_TYPE_MASK>(target, mask);
    let expected = if solution.is_some() { 1 } else { limit };
    if solver.attempted_nonces() != expected {
        return Err(format!(
            "incorrect work accounting: got {}, expected {expected}, hit={}",
            solver.attempted_nonces(),
            solution.is_some()
        ));
    }
    Ok(solution)
}

fn execute(
    mode: Mode,
    msg: &SingleBlockMessage,
    limit: u64,
    target: u64,
    mask: u64,
) -> Result<Option<(u64, [u32; 8])>, String> {
    match mode {
        Mode::ShaNi => run_solver::<challenge_solvers::solver::sha_ni::SingleBlockSolver>(
            msg, limit, target, mask,
        ),
        Mode::Simd if std::arch::is_x86_feature_detected!("avx512f") => {
            run_solver::<challenge_solvers::solver::x86::Avx512SingleBlockSolver>(
                msg, limit, target, mask,
            )
        }
        Mode::Simd => run_solver::<challenge_solvers::solver::x86::Avx2SingleBlockSolver>(
            msg, limit, target, mask,
        ),
    }
}

fn check_oracle(worker: usize) -> Result<(), String> {
    let p = prefix(worker);
    let message = SingleBlockMessage::new(&p, 0).ok_or("invalid message")?;
    if message.digit_index != 42 || message.no_trailing_zeros || message.nonce_addend != 0 {
        return Err("fixture layout changed".into());
    }
    let mut text = [0_u8; 51];
    text[..42].copy_from_slice(&p);
    text[42..].copy_from_slice(b"100000000");
    let digest = Sha256::digest(text);
    let expected: [u32; 8] = core::array::from_fn(|i| {
        u32::from_be_bytes(digest[4 * i..4 * i + 4].try_into().expect("4 bytes"))
    });
    if worker == 0 && format!("{:08x}{:08x}", expected[0], expected[1]) != SHA256_FIRST {
        return Err("known SHA-256 fixture changed".into());
    }
    for mode in [Mode::ShaNi, Mode::Simd] {
        let result = execute(mode, &message, 1, 0, 0)?;
        let (nonce, words) = result.ok_or("did not find trivial target")?;
        if nonce != 100_000_000 || words != expected {
            return Err(format!(
                "full SHA-256 oracle mismatch worker={worker} mode={} nonce={nonce}",
                mode.label()
            ));
        }
    }
    Ok(())
}

fn pin(cpu: usize) -> Result<(), String> {
    // SAFETY: fully initialized cpu_set_t with checked CPU bound; pid=0
    // refers to the calling Linux thread.
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        if cpu >= libc::CPU_SETSIZE as usize {
            return Err(format!("CPU {cpu} exceeds CPU_SETSIZE"));
        }
        libc::CPU_ZERO(&mut set);
        libc::CPU_SET(cpu, &mut set);
        if libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set) != 0 {
            return Err(format!(
                "affinity CPU {cpu}: {}",
                std::io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}

fn trial(mode: Mode, cpus: &[usize], per_worker: u64, warmup: u64) -> Result<(u64, u128), String> {
    let barrier = Arc::new(Barrier::new(cpus.len() + 1));
    let (duration, attempts) = thread::scope(|scope| -> Result<(u128, u64), String> {
        let mut workers = Vec::with_capacity(cpus.len());
        for (worker_id, &cpu) in cpus.iter().enumerate() {
            let barrier = Arc::clone(&barrier);
            workers.push(scope.spawn(move || {
                let ready = (|| -> Result<SingleBlockMessage, String> {
                    pin(cpu)?;
                    let message = SingleBlockMessage::new(&prefix(worker_id), 0)
                        .ok_or("message setup failed")?;
                    if execute(mode, &message, warmup, TARGET, u64::MAX)?.is_some() {
                        return Err("unexpected SHA-256 warmup target hit".into());
                    }
                    Ok(message)
                })();
                barrier.wait();
                let message = ready?;
                let result = execute(mode, &message, per_worker, TARGET, u64::MAX)?;
                if result.is_some() {
                    return Err("unexpected SHA-256 benchmark target hit".into());
                }
                Ok::<u64, String>(per_worker)
            }));
        }
        let start = Instant::now();
        barrier.wait();
        let mut total = 0_u64;
        let mut first_error = None;
        for worker in workers {
            match worker.join() {
                Ok(Ok(count)) => total += count,
                Ok(Err(err)) => {
                    first_error.get_or_insert(err);
                }
                Err(_) => {
                    first_error.get_or_insert("worker panic".into());
                }
            }
        }
        let elapsed = start.elapsed().as_nanos();
        if let Some(err) = first_error {
            return Err(err);
        }
        Ok((elapsed, total))
    })?;
    let expected = per_worker * u64::try_from(cpus.len()).expect("threads fits");
    if attempts != expected {
        return Err(format!(
            "aggregate attempted count mismatch: {attempts} vs {expected}"
        ));
    }
    Ok((attempts, duration))
}

fn arg(name: &str, default: u64) -> Result<u64, String> {
    let key = format!("--{name}");
    let items: Vec<String> = std::env::args().collect();
    match items.windows(2).find(|x| x[0] == key) {
        Some(args) => args[1].parse().map_err(|e| format!("{name}: {e}")),
        None => Ok(default),
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let cpu_string = args
        .windows(2)
        .find(|a| a[0] == "--cpus")
        .map_or("0,1,2,3,4,5,6,7", |a| a[1].as_str());
    let cpus: Vec<usize> = cpu_string
        .split(',')
        .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if cpus.len() != 8 {
        return Err("pass exactly 8 CPU slots".into());
    }
    if !std::arch::is_x86_feature_detected!("sha")
        || !std::arch::is_x86_feature_detected!("sse4.1")
        || !std::arch::is_x86_feature_detected!("avx2")
    {
        return Err("the requested SHA-NI/portable-SIMD backends are unavailable".into());
    }
    let per_worker = arg("iterations", 80_000_000)?;
    let warmup = arg("warmup", 1_000_000)?;
    let samples = arg("samples", 5)?;
    if per_worker == 0
        || per_worker > 160_000_000
        || warmup > 160_000_000
        || samples == 0
        || samples > 20
    {
        return Err("invalid workload dimensions".into());
    }
    for worker in 0..8 {
        check_oracle(worker)?;
    }
    let simd_lanes = if std::arch::is_x86_feature_detected!("avx512f") {
        16
    } else {
        8
    };
    println!(
        "META source=3776d8fbce45ae4641719d9ae40f84578be7c228 sha_ni=4 simd={simd_lanes} cpus={cpu_string} work_per_worker={per_worker} warmup={warmup} samples={samples}"
    );
    println!("ORACLE PASS workers=8 first_sha256={SHA256_FIRST}");

    for n in [1, 2, 4, 8] {
        for sample in 0..samples {
            let order = if sample & 1 == 0 {
                [Mode::ShaNi, Mode::Simd]
            } else {
                [Mode::Simd, Mode::ShaNi]
            };
            for mode in order {
                let (attempts, elapsed) = trial(mode, &cpus[..n], per_worker, warmup)?;
                let rate = (attempts as f64) * 1e9 / elapsed as f64;
                println!(
                    "SAMPLE backend={} threads={n} rep={sample} attempts={attempts} elapsed_ns={elapsed} hps={rate:.3}",
                    mode.label()
                );
            }
        }
    }
    Ok(())
}
