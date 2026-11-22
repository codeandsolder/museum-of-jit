use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::config::{Backend, Plan, PythonEngine};

#[derive(Debug, Serialize)]
pub(crate) struct Sample {
    pub hashes_per_second: f64,
    pub sink: String,
}

#[derive(Default, Debug, Serialize)]
pub(crate) struct Evidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    simd_lanes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jit_batch_internal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    llvm_ir_optimization: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    llvm_codegen: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jit_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jit_executors: Option<u32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Measurements {
    pub check: String,
    pub build_us: Option<f64>,
    pub first_call_us: Option<f64>,
    pub code_bytes: Option<u64>,
    pub samples: Vec<Sample>,
}

fn optional_field<'a>(text: &'a str, name: &str) -> Result<Option<&'a str>> {
    let mut fields = text.split_ascii_whitespace();
    let mut found = None;
    while let Some(field) = fields.next() {
        if field == name {
            let value = fields
                .next()
                .with_context(|| format!("missing value for {name}"))?;
            ensure!(found.replace(value).is_none(), "duplicate field {name}");
        }
    }
    Ok(found)
}

fn field<'a>(text: &'a str, name: &str) -> Result<&'a str> {
    optional_field(text, name)?.with_context(|| format!("missing {name}"))
}

fn number(value: &str) -> Result<f64> {
    let number: f64 = value.parse()?;
    ensure!(
        number.is_finite() && number >= 0.0,
        "invalid measurement {value}"
    );
    Ok(number)
}

fn hex64(value: &str) -> Result<String> {
    ensure!(
        value.len() == 16 && value.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid digest {value}"
    );
    Ok(value.to_ascii_lowercase())
}

fn one_line<'a>(text: &'a str, prefix: &str) -> Result<&'a str> {
    let mut lines = text.lines().filter(|line| line.starts_with(prefix));
    let line = lines.next().with_context(|| format!("missing {prefix}"))?;
    ensure!(lines.next().is_none(), "duplicate {prefix}");
    Ok(line)
}

fn exact_line(text: &str, prefix: &str, expected: &str) -> Result<()> {
    let actual = one_line(text, prefix)?;
    ensure!(
        actual == expected,
        "wrong {prefix}: expected {expected:?}, got {actual:?}"
    );
    Ok(())
}

fn sample(line: &str, rpython: bool, iterations: u32) -> Result<Sample> {
    let parts: Vec<_> = line.split_ascii_whitespace().collect();
    let (rate, sink) = if rpython {
        let ["SAMPLE_NS", nanos, "SINK", sink] = parts.as_slice() else {
            anyhow::bail!("invalid RPython sample {line}");
        };
        (f64::from(iterations) / (number(nanos)? / 1e9), *sink)
    } else {
        let ["RATE", rate, sink] = parts.as_slice() else {
            anyhow::bail!("invalid sample {line}");
        };
        (
            number(rate)?,
            sink.strip_prefix("sink=").context("missing sample sink")?,
        )
    };
    ensure!(rate.is_finite() && rate > 0.0, "invalid sample rate");
    Ok(Sample {
        hashes_per_second: rate,
        sink: hex64(sink)?,
    })
}

fn measurements(
    text: &str,
    plan: Plan,
    expected_check: &str,
    rpython: bool,
) -> Result<Measurements> {
    exact_line(text, "BENCHMARK_DIMENSIONS", &plan.attestation())?;
    let check = hex64(field(text, "CHECK")?)?;
    ensure!(
        check == expected_check,
        "wrong CHECK {check}, expected {expected_check}"
    );
    let prefix = if rpython { "SAMPLE_NS " } else { "RATE " };
    let samples = text
        .lines()
        .filter(|line| line.starts_with(prefix))
        .map(|line| sample(line, rpython, plan.iterations.get()))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        samples.len() == usize::try_from(plan.samples.get())?,
        "wrong sample count: {}",
        samples.len()
    );
    let first_call_us = if rpython {
        Some(number(field(text, "FIRST_NS")?)? / 1000.0)
    } else {
        optional_field(text, "FIRST_US")?.map(number).transpose()?
    };
    Ok(Measurements {
        check,
        build_us: optional_field(text, "BUILD_US")?.map(number).transpose()?,
        first_call_us,
        code_bytes: optional_field(text, "CODE")?.map(str::parse).transpose()?,
        samples,
    })
}

fn lanes(text: &str, engine: &str, mode: &str) -> Result<u32> {
    let line = one_line(text, "ENGINE ")?;
    let parts: Vec<_> = line.split_ascii_whitespace().collect();
    let [
        "ENGINE",
        actual_engine,
        "MODE",
        actual_mode,
        "VECTOR_LANES",
        lanes,
    ] = parts.as_slice()
    else {
        anyhow::bail!("invalid SIMD attestation {line}");
    };
    ensure!(
        *actual_engine == engine && *actual_mode == mode,
        "wrong SIMD backend {line}"
    );
    let lanes: u32 = lanes.parse()?;
    ensure!(lanes > 0, "invalid SIMD lane count");
    Ok(lanes)
}

fn evidence(backend: &Backend, text: &str) -> Result<Evidence> {
    let mut evidence = Evidence::default();
    match backend {
        Backend::RustChallenge => {
            evidence.simd_lanes = Some(lanes(text, "rust-challenge", "simd")?);
            evidence.backend = Some("production-portable-simd");
        }
        Backend::Dotnet {
            challenge: Some(mode),
            ..
        } => {
            evidence.simd_lanes = Some(lanes(text, "dotnet", mode.name())?);
            evidence.backend = Some(mode.name());
        }
        Backend::Llvm { policy } => {
            exact_line(text, "MODE ", "MODE jit-batch")?;
            exact_line(text, "IR_OPT ", &format!("IR_OPT {}", policy.name()))?;
            exact_line(text, "ENGINE ", "ENGINE llvm-orc VERSION 21.1.8")?;
            evidence.jit_batch_internal = Some(true);
            evidence.llvm_ir_optimization = Some(policy.name());
            evidence.llvm_codegen = Some("Aggressive");
        }
        Backend::Cranelift { batch } => {
            exact_line(
                text,
                "MODE ",
                if *batch {
                    "MODE jit-batch"
                } else {
                    "MODE direct"
                },
            )?;
            evidence.jit_batch_internal = Some(*batch);
        }
        Backend::Python {
            engine: PythonEngine::Cpython,
        } => {
            ensure!(field(text, "JIT")? == "true", "CPython JIT was not enabled");
            let count: u32 = field(text, "JIT_EXECUTORS")?.parse()?;
            ensure!(count > 0, "CPython JIT installed no tier-2 executors");
            evidence.jit_enabled = Some(true);
            evidence.jit_executors = Some(count);
        }
        _ => {}
    }
    Ok(evidence)
}

pub(crate) fn validate(
    backend: &Backend,
    text: &str,
    plan: Plan,
    expected_check: &str,
) -> Result<(Measurements, Evidence)> {
    let parsed = measurements(
        text,
        plan,
        expected_check,
        matches!(backend, Backend::Rpython),
    )?;
    let hit_only = match backend {
        Backend::RustChallenge => true,
        Backend::Dotnet {
            challenge: Some(mode),
            ..
        } => mode.hit_only(),
        _ => false,
    };
    if hit_only {
        let expected = format!("{:016x}", plan.iterations);
        ensure!(
            parsed.samples.iter().all(|sample| sample.sink == expected),
            "wrong hit-only attempted count"
        );
    }
    Ok((parsed, evidence(backend, text)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ChallengeMode, IrPolicy};
    use std::num::NonZeroU32;

    fn plan() -> Result<Plan> {
        Ok(Plan {
            warmup: 3,
            iterations: NonZeroU32::new(128).context("iterations")?,
            samples: NonZeroU32::new(2).context("samples")?,
            timeout: 10.0,
        })
    }

    const CHECK: &str = "fd5d450b767035c5";
    const HASH: &str = "BENCHMARK_DIMENSIONS v1 3 128 2\nCHECK fd5d450b767035c5\nRATE 11 sink=d7a1b517d68af6bb\nRATE 12 sink=d7a1b517d68af6bb\n";

    #[test]
    fn reject_missing_wrong_duplicate_and_unknown_dimensions() -> Result<()> {
        let plan = plan()?;
        let backend = Backend::RustDynasm;
        validate(&backend, HASH, plan, CHECK)?;
        for wrong in [
            HASH.replace("BENCHMARK_DIMENSIONS v1 3 128 2\n", ""),
            HASH.replace("3 128 2", "3 127 2"),
            format!("{HASH}BENCHMARK_DIMENSIONS v1 3 128 2\n"),
            HASH.replace("v1", "v2"),
            HASH.replace("RATE 11", "RATE 0"),
        ] {
            assert!(validate(&backend, &wrong, plan, CHECK).is_err());
        }
        assert!(validate(&backend, HASH, plan, "b4b920f70c0de8df").is_err());
        Ok(())
    }

    #[test]
    fn llvm_policies_require_actual_mode_and_exact_version() -> Result<()> {
        let plan = plan()?;
        for policy in [IrPolicy::O0, IrPolicy::O1, IrPolicy::O2, IrPolicy::O3] {
            let text = format!(
                "{HASH}ENGINE llvm-orc VERSION 21.1.8\nMODE jit-batch\nIR_OPT {}\n",
                policy.name()
            );
            let backend = Backend::Llvm { policy };
            validate(&backend, &text, plan, CHECK)?;
            for wrong in [
                text.replace("jit-batch", "direct"),
                text.replace("21.1.8", "23.0"),
                text.replace(policy.name(), "O99"),
            ] {
                assert!(validate(&backend, &wrong, plan, CHECK).is_err());
            }
        }
        Ok(())
    }

    #[test]
    fn challenge_modes_verify_hit_counts_and_attestation() -> Result<()> {
        let plan = plan()?;
        let hash = HASH
            .replace(CHECK, "b4b920f70c0de8df")
            .replace("d7a1b517d68af6bb", "0000000000000080");
        for mode in [
            ChallengeMode::Portable,
            ChallengeMode::Optimized,
            ChallengeMode::Match,
            ChallengeMode::MatchOptimized,
            ChallengeMode::Scalar,
            ChallengeMode::Bcl,
        ] {
            let text = format!("{hash}ENGINE dotnet MODE {} VECTOR_LANES 16\n", mode.name());
            let backend = Backend::Dotnet {
                project: "ChallengeSha256".into(),
                challenge: Some(mode),
            };
            validate(&backend, &text, plan, "b4b920f70c0de8df")?;
            assert!(
                validate(
                    &backend,
                    &text.replace(mode.name(), "unknown"),
                    plan,
                    "b4b920f70c0de8df"
                )
                .is_err()
            );
            if mode.hit_only() {
                assert!(
                    validate(
                        &backend,
                        &text.replace("0000000000000080", "0000000000000081"),
                        plan,
                        "b4b920f70c0de8df"
                    )
                    .is_err()
                );
            }
        }
        let text = format!("{hash}ENGINE rust-challenge MODE simd VECTOR_LANES 16\n");
        validate(&Backend::RustChallenge, &text, plan, "b4b920f70c0de8df")?;
        assert!(
            validate(
                &Backend::RustChallenge,
                &text.replace("0000000000000080", "0000000000000081"),
                plan,
                "b4b920f70c0de8df"
            )
            .is_err()
        );
        Ok(())
    }
}
