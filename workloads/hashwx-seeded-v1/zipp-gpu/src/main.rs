mod cuda;
mod direct;
mod shader;

use std::{error::Error, time::Instant};

use hashwx::{HashWx, HashX, HashXProgram, generate};
use serde_json::{Value, json};
use zipp_gpu::{GpuHost, GpuOptions};
use zipp_vm::embed::JsValue;

const SEED: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];

#[derive(Clone, Copy)]
enum Algorithm {
    HashX,
    HashWx,
}

impl Algorithm {
    const fn max_batch(self) -> usize {
        match self {
            Self::HashX => 65536,
            Self::HashWx => 4096,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::HashX => "hashx",
            Self::HashWx => "hashwx",
        }
    }
}

struct Case {
    shader: String,
    expected: Vec<u64>,
    cpu_build_ms: f64,
    cpu_times_ms: Vec<f64>,
}

fn milliseconds(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn inputs(count: usize) -> Vec<u64> {
    let mut values = vec![0, 1, u64::MAX, 1 << 32, (1 << 63) - 1, 1 << 63, 123_456];
    let mut state = 0x9124_afe1_2473_dcb3_u64;
    while values.len() < count {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        values.push(state);
    }
    values.truncate(count);
    values
}

fn seed_from_hex(hex: &str) -> Result<[u8; 32], Box<dyn Error>> {
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("seed must contain exactly 64 hex digits".into());
    }
    let mut seed = [0; 32];
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)?;
    }
    Ok(seed)
}

fn cpu_samples(
    salts: &[u64],
    expected: &[u64],
    samples: usize,
    hash: impl Fn(u64) -> u64,
) -> Result<Vec<f64>, Box<dyn Error>> {
    let mut output = vec![0; salts.len()];
    let mut times = Vec::with_capacity(samples);
    for step in 0..=samples {
        let start = Instant::now();
        for (&salt, value) in salts.iter().zip(&mut output) {
            *value = hash(salt);
        }
        let elapsed = milliseconds(start);
        if output != expected {
            return Err("native CPU JIT differs from portable reference".into());
        }
        if step != 0 {
            times.push(elapsed);
        }
    }
    Ok(times)
}

fn prepare(
    algorithm: Algorithm,
    seed: &[u8; 32],
    salts: &[u64],
    samples: usize,
) -> Result<Case, Box<dyn Error>> {
    let (shader, expected, cpu_build_ms, cpu_times_ms) = match algorithm {
        Algorithm::HashWx => {
            let program = generate(seed);
            let reference = HashWx::interpret_program(&program);
            let expected: Vec<_> = salts.iter().map(|&salt| reference.hash(salt)).collect();
            let start = Instant::now();
            let jit = HashWx::compile_program(&program)?;
            let build = milliseconds(start);
            let times = cpu_samples(salts, &expected, samples, |salt| jit.hash(salt))?;
            (shader::hashwx(&program)?, expected, build, times)
        }
        Algorithm::HashX => {
            let program = HashXProgram::generate(seed)?;
            let reference = HashX::interpret_program(&program);
            let expected: Vec<_> = salts.iter().map(|&salt| reference.hash(salt)).collect();
            let start = Instant::now();
            let jit = HashX::compile_program(&program)?;
            let build = milliseconds(start);
            let times = cpu_samples(salts, &expected, samples, |salt| jit.hash(salt))?;
            (shader::hashx(&program)?, expected, build, times)
        }
    };
    Ok(Case {
        shader,
        expected,
        cpu_build_ms,
        cpu_times_ms,
    })
}

fn gpu_report(
    host: &mut GpuHost,
    case: &Case,
    salts: &[u64],
    algorithm: Algorithm,
    samples: usize,
) -> Result<Value, Box<dyn Error>> {
    let words: Vec<u32> = salts
        .iter()
        .flat_map(|salt| {
            let [lo0, lo1, lo2, lo3, hi0, hi1, hi2, hi3] = salt.to_le_bytes();
            [
                u32::from_le_bytes([lo0, lo1, lo2, lo3]),
                u32::from_le_bytes([hi0, hi1, hi2, hi3]),
            ]
        })
        .collect();
    let program = format!(
        "{}\nvar benchResult = null; runCase({}, {}, {samples}, {}).then(\n\
         v => {{benchResult = JSON.stringify(v);}},\n\
         e => {{benchResult = 'ERROR ' + String(e && e.stack || e);}}); null",
        include_str!("bench.js"),
        serde_json::to_string(&case.shader)?,
        serde_json::to_string(&words)?,
        matches!(algorithm, Algorithm::HashWx)
    );
    host.eval(&program)?;
    let JsValue::String(result) = host.eval("benchResult")? else {
        return Err("Zipp GPU case did not finish".into());
    };
    let mut report: Value = serde_json::from_str(&result)
        .map_err(|e| format!("{e}: {}", result.chars().take(512).collect::<String>()))?;
    let values = report["values"].as_array().ok_or("missing GPU outputs")?;
    if values.len() != salts.len() * 2 {
        return Err("wrong GPU output length".into());
    }
    for ((pair, &reference), &salt) in values
        .as_chunks::<2>()
        .0
        .iter()
        .zip(&case.expected)
        .zip(salts)
    {
        let lo = pair[0]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid GPU low word")?;
        let hi = pair[1]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid GPU high word")?;
        let actual = u64::from(lo) | (u64::from(hi) << 32);
        if actual != reference {
            return Err(format!(
                "hash mismatch at salt {salt:016x}: GPU {actual:016x}, CPU {reference:016x}"
            )
            .into());
        }
    }
    let object = report.as_object_mut().ok_or("missing report")?;
    object.remove("values");
    object.insert("algorithm".into(), json!(algorithm.name()));
    object.insert("verified".into(), json!(salts.len()));
    object.insert("hardware".into(), json!(host.adapter().describe()));
    object.insert("hashes_per_sample".into(), json!(salts.len()));
    object.insert("cpu_build_ms".into(), json!(case.cpu_build_ms));
    object.insert("cpu_times_ms".into(), json!(case.cpu_times_ms));
    Ok(report)
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let export = args.first().is_some_and(|arg| arg == "export");
    let dest = if export {
        if args.len() < 3 {
            return Err("export requires a directory and workload arguments".into());
        }
        args.remove(0);
        Some(std::path::PathBuf::from(args.remove(0)))
    } else {
        None
    };
    let direct = args.first().is_some_and(|arg| arg == "direct");
    if direct {
        args.remove(0);
    }
    let (algorithm, count, samples, backend, seed) = match args.as_slice() {
        [algorithm, count, samples, backend] => (algorithm, count, samples, backend, SEED),
        [algorithm, count, samples, backend, seed] => {
            (algorithm, count, samples, backend, seed_from_hex(seed)?)
        }
        _ => return Err(
            "usage: museum-zipp-gpu [direct | export DIR] hashx|hashwx COUNT SAMPLES auto|vulkan|dx12|metal [SEED_HEX]"
                .into(),
        ),
    };
    let algorithm = match algorithm.as_str() {
        "hashwx" => Algorithm::HashWx,
        "hashx" => Algorithm::HashX,
        _ => return Err("unknown algorithm".into()),
    };
    let count: usize = count.parse()?;
    let samples: usize = samples.parse()?;
    if count == 0 || count > algorithm.max_batch() || samples == 0 || samples > 100 {
        return Err("batch dimensions out of range".into());
    }
    let salts = inputs(count);
    let case = prepare(algorithm, &seed, &salts, samples)?;
    if let Some(dest) = dest {
        cuda::export(algorithm, &seed, &salts, &case, &dest)?;
        return Ok(());
    }
    if direct {
        let mut report = direct::report(&case, &salts, algorithm, samples, backend)?;
        report["seed"] = json!(seed);
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    let mut host = GpuHost::open(&GpuOptions {
        backend: (backend != "auto").then(|| backend.clone()),
        ..GpuOptions::default()
    })?
    .ok_or("no hardware GPU adapter")?;
    let mut report = gpu_report(&mut host, &case, &salts, algorithm, samples)?;
    report["seed"] = json!(seed);
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn main() -> std::process::ExitCode {
    let outcome = std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(|| run().map_err(|error| error.to_string()))
        .map_err(|error| error.to_string())
        .and_then(|thread| {
            thread
                .join()
                .map_err(|_| "benchmark thread panicked".to_owned())
        })
        .flatten();
    if let Err(error) = outcome {
        eprintln!("{error}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
