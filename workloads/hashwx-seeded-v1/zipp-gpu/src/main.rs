mod shader;

use std::error::Error;

use hashwx::{HashWx, HashX, HashXProgram, generate};
use serde_json::{Value, json};
use zipp_gpu::{GpuHost, GpuOptions};
use zipp_vm::embed::JsValue;

const SEED: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];

fn inputs(count: usize) -> Vec<u64> {
    let mut values = vec![0, 1, u64::MAX, 1 << 32, (1 << 63) - 1, 1 << 63, 123456];
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

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [algorithm, count, samples, backend] = args.as_slice() else {
        return Err(
            "usage: museum-zipp-gpu hashx|hashwx COUNT SAMPLES auto|vulkan|dx12|metal".into(),
        );
    };
    let count: usize = count.parse()?;
    let samples: usize = samples.parse()?;
    let memory = match algorithm.as_str() {
        "hashwx" => true,
        "hashx" => false,
        _ => return Err("unknown algorithm".into()),
    };
    if count == 0 || count > if memory { 4096 } else { 65536 } || samples == 0 || samples > 100 {
        return Err("batch dimensions out of range".into());
    }
    let salts = inputs(count);
    let (source, expected): (String, Vec<u64>) = if memory {
        let program = generate(&SEED);
        let reference = HashWx::interpret_program(&program);
        (
            shader::hashwx(&program),
            salts.iter().map(|&salt| reference.hash(salt)).collect(),
        )
    } else {
        let program = HashXProgram::generate(&SEED)?;
        let reference = HashX::interpret_program(&program);
        (
            shader::hashx(&program),
            salts.iter().map(|&salt| reference.hash(salt)).collect(),
        )
    };
    let words: Vec<u32> = salts
        .iter()
        .flat_map(|salt| {
            let bytes = salt.to_le_bytes();
            [
                u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                u32::from_le_bytes(bytes[4..].try_into().unwrap()),
            ]
        })
        .collect();
    let mut host = GpuHost::open(&GpuOptions {
        backend: (backend != "auto").then(|| backend.clone()),
        ..GpuOptions::default()
    })?
    .ok_or("no hardware GPU adapter")?;
    let program = format!(
        "{}\nvar benchResult = null; runCase({}, {}, {}, {}).then(\n\
         v => {{benchResult = JSON.stringify(v);}},\n\
         e => {{benchResult = 'ERROR ' + String(e && e.stack || e);}}); null",
        include_str!("bench.js"),
        serde_json::to_string(&source)?,
        serde_json::to_string(&words)?,
        samples,
        memory
    );
    host.eval(&program)?;
    let JsValue::String(result) = host.eval("benchResult")? else {
        return Err("Zipp GPU case did not finish".into());
    };
    let mut report: Value = serde_json::from_str(&result)
        .map_err(|e| format!("{e}: {}", result.chars().take(512).collect::<String>()))?;
    let values = report["values"].as_array().ok_or("missing GPU outputs")?;
    if values.len() != count * 2 {
        return Err("wrong GPU output length".into());
    }
    for (index, (pair, reference)) in values.chunks_exact(2).zip(expected).enumerate() {
        let lo = pair[0].as_u64().ok_or("non-integer GPU output")?;
        let hi = pair[1].as_u64().ok_or("non-integer GPU output")?;
        let actual = lo | (hi << 32);
        if actual != reference {
            return Err(format!(
                "hash mismatch at salt {:016x}: GPU {actual:016x}, CPU {reference:016x}",
                salts[index]
            )
            .into());
        }
    }
    let object = report.as_object_mut().ok_or("missing report")?;
    object.remove("values");
    object.insert("algorithm".into(), json!(algorithm));
    object.insert("verified".into(), json!(count));
    object.insert("hardware".into(), json!(host.adapter().describe()));
    object.insert("hashes_per_sample".into(), json!(count));
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn main() {
    let outcome = std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(|| run().map_err(|error| error.to_string()))
        .expect("benchmark thread")
        .join()
        .expect("benchmark thread panicked");
    if let Err(error) = outcome {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
