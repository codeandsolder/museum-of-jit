use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, ensure};
use chrono::{SecondsFormat, Utc};
use nix::{
    sched::{CpuSet, sched_getaffinity},
    sys::utsname::uname,
    unistd::{Pid, gethostname},
};
use serde::{Deserialize, Serialize};

use crate::{
    backend::{prepare, sha256},
    config::{Implementation, ImplementationMode, Plan, RuntimeConfig, Workload},
    process::{CommandRecord, Session},
    protocol::{self, Evidence, Measurements, Sample},
};

#[derive(Debug, Serialize)]
pub(crate) struct Host {
    hostname: String,
    arch: String,
    cpu_model: String,
    cpu_family: Option<String>,
    cpu_model_id: Option<String>,
    cpu_affinity: Option<Vec<usize>>,
    os: String,
    kernel: String,
    cpu_flags: Vec<String>,
}

impl Host {
    pub fn read() -> Result<Self> {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
        let first = cpuinfo.split("\n\n").next().context("empty cpuinfo")?;
        let fields: BTreeMap<_, _> = first
            .lines()
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.trim(), value.trim()))
            .collect();
        let kernel = uname()?;
        let os = fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("PRETTY_NAME=").and_then(shlex::split))
                    .map(|words| words.join(" "))
            })
            .unwrap_or_else(|| kernel.sysname().to_string_lossy().into_owned());
        let affinity = sched_getaffinity(Pid::from_raw(0)).ok().map(|cpus| {
            (0..CpuSet::count())
                .filter(|&index| cpus.is_set(index).unwrap_or(false))
                .collect()
        });
        Ok(Self {
            hostname: gethostname()?.to_string_lossy().into_owned(),
            arch: kernel.machine().to_string_lossy().into_owned(),
            cpu_model: fields
                .get("model name")
                .or_else(|| fields.get("Processor"))
                .copied()
                .unwrap_or("unknown")
                .into(),
            cpu_family: fields.get("cpu family").map(|value| (*value).into()),
            cpu_model_id: fields.get("model").map(|value| (*value).into()),
            cpu_affinity: affinity,
            os,
            kernel: kernel.release().to_string_lossy().into_owned(),
            cpu_flags: fields
                .get("flags")
                .or_else(|| fields.get("Features"))
                .map_or_else(Vec::new, |value| {
                    value.split_ascii_whitespace().map(str::to_owned).collect()
                }),
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Corpus {
    id: &'static str,
    pub corpus_sha256: String,
    #[serde(skip)]
    pub path: PathBuf,
    #[serde(skip)]
    pub expected_check: String,
}

#[derive(Deserialize)]
struct Fixture {
    expected_check: String,
}

impl Corpus {
    pub fn load(root: &Path, workload: Workload) -> Result<Self> {
        let directory = root.join("workloads").join(workload.id());
        let (path, expected_check) = match workload {
            Workload::HashwxFixedV1 => (directory.join("program.json"), "fd5d450b767035c5".into()),
            Workload::ChallengeSha256V1 => {
                let path = directory.join("fixture.json");
                let fixture: Fixture = serde_json::from_slice(&fs::read(&path)?)?;
                (path, fixture.expected_check)
            }
        };
        ensure!(
            expected_check.len() == 16 && expected_check.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid corpus correctness probe"
        );
        Ok(Self {
            id: workload.id(),
            corpus_sha256: sha256(&path)?,
            path,
            expected_check,
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Revision {
    pub commit: String,
    pub dirty: bool,
}

#[derive(Serialize)]
struct ImplementationRecord<'a> {
    id: &'a str,
    mode: ImplementationMode,
    source: &'a Path,
    revision: &'a str,
    working_tree_dirty: bool,
}

#[derive(Serialize)]
struct RuntimeRecord<'a> {
    name: &'a str,
    version: String,
    command: Vec<String>,
    commands: Vec<CommandRecord>,
    jit_configuration: BTreeMap<&'static str, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    binary_sha256: Option<String>,
    #[serde(flatten)]
    evidence: Evidence,
}

impl<'a> RuntimeRecord<'a> {
    fn new(name: &'a str) -> Self {
        let jit_configuration = [
            "DOTNET_MaxVectorTBitWidth",
            "DOTNET_PreferredVectorBitWidth",
            "DOTNET_EnableAVX2",
            "DOTNET_EnableAVX512F",
            "DOTNET_EnableHWIntrinsic",
            "COMPlus_TieredPGO",
            "COMPlus_TieredCompilation",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key, value)))
        .collect();
        Self {
            name,
            version: "unavailable".into(),
            command: Vec::new(),
            commands: Vec::new(),
            jit_configuration,
            binary_sha256: None,
            evidence: Evidence::default(),
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Success,
    Failed,
    Invalid,
    Timeout,
}

#[derive(Serialize)]
pub(crate) struct Row<'a> {
    schema_version: u32,
    run_id: String,
    timestamp_utc: String,
    host: &'a Host,
    workload: &'a Corpus,
    implementation: ImplementationRecord<'a>,
    runtime: RuntimeRecord<'a>,
    warmup_iterations: u32,
    sample_iterations: u32,
    sample_count: u32,
    samples: Vec<Sample>,
    status: Status,
    exit_code: Option<i32>,
    error: Option<String>,
    build_us: Option<f64>,
    first_call_us: Option<f64>,
    code_bytes: Option<u64>,
    check: Option<String>,
    stdout: String,
    stderr: String,
    elapsed_seconds: f64,
}

impl Row<'_> {
    pub fn succeeded(&self) -> bool {
        matches!(self.status, Status::Success)
    }
    fn accept(&mut self, parsed: Measurements, evidence: Evidence) {
        self.status = Status::Success;
        self.check = Some(parsed.check);
        self.build_us = parsed.build_us;
        self.first_call_us = parsed.first_call_us;
        self.code_bytes = parsed.code_bytes;
        self.samples = parsed.samples;
        self.runtime.evidence = evidence;
    }

    fn report(&self) {
        if self.succeeded() {
            let mut rates: Vec<_> = self
                .samples
                .iter()
                .map(|sample| sample.hashes_per_second)
                .collect();
            rates.sort_by(f64::total_cmp);
            let median = match rates.len() {
                0 => 0.0,
                length if length % 2 == 1 => rates[length / 2],
                length => f64::midpoint(rates[length / 2 - 1], rates[length / 2]),
            };
            eprintln!(
                "{:24} median={median:12.3} H/s samples={}",
                self.implementation.id,
                self.samples.len()
            );
        } else {
            eprintln!(
                "{:24} {}",
                self.implementation.id,
                self.error.as_deref().unwrap_or("benchmark failed")
            );
        }
    }
}

/// Shared immutable context. Each exhibit receives an independent session.
pub(crate) struct RunContext<'a> {
    pub root: &'a Path,
    pub config: &'a RuntimeConfig,
    pub host: &'a Host,
    pub corpus: &'a Corpus,
    pub revision: &'a Revision,
    pub plan: Plan,
    pub timeout: Duration,
    pub cancelled: &'a AtomicBool,
}

impl<'a> RunContext<'a> {
    pub fn run(&self, implementation: &'a Implementation) -> Result<Row<'a>> {
        let started = Instant::now();
        let mut session = Session::new(self.root, self.timeout, self.cancelled)?;
        let mut row = Row {
            schema_version: 1,
            run_id: uuid::Uuid::new_v4().to_string(),
            timestamp_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            host: self.host,
            workload: self.corpus,
            implementation: ImplementationRecord {
                id: &implementation.id,
                mode: implementation.mode,
                source: &implementation.source,
                revision: &self.revision.commit,
                working_tree_dirty: self.revision.dirty,
            },
            runtime: RuntimeRecord::new(&implementation.runtime),
            warmup_iterations: self.plan.warmup,
            sample_iterations: self.plan.iterations.get(),
            sample_count: self.plan.samples.get(),
            samples: Vec::new(),
            status: Status::Failed,
            exit_code: None,
            error: None,
            build_us: None,
            first_call_us: None,
            code_bytes: None,
            check: None,
            stdout: String::new(),
            stderr: String::new(),
            elapsed_seconds: 0.0,
        };
        match self.execute(&mut session, implementation, &mut row) {
            Ok(()) => {}
            Err(error) => {
                if session.timed_out {
                    row.status = Status::Timeout;
                }
                row.error = Some(format!("{error:#}"));
            }
        }
        row.exit_code = session.last_exit;
        row.stderr = std::mem::take(&mut session.diagnostics);
        row.runtime.command = session
            .records
            .iter()
            .find(|record| record.stage == "benchmark")
            .map_or_else(Vec::new, |record| record.argv.clone());
        row.runtime.commands = std::mem::take(&mut session.records);
        row.elapsed_seconds = started.elapsed().as_secs_f64();
        row.report();
        Ok(row)
    }

    fn execute(
        &self,
        session: &mut Session<'_>,
        implementation: &Implementation,
        row: &mut Row<'_>,
    ) -> Result<()> {
        let prepared = prepare(
            session,
            self.config,
            implementation,
            &self.corpus.path,
            self.plan,
        )?;
        row.runtime.version = prepared.version;
        row.runtime.binary_sha256 = prepared.binary_sha256;
        let output = session.invoke(prepared.command, "benchmark")?;
        row.stdout = output.stdout;
        ensure!(
            output.code == 0,
            "benchmark exited with status {}",
            output.code
        );
        row.status = Status::Invalid;
        let (parsed, evidence) = protocol::validate(
            &implementation.backend,
            &row.stdout,
            self.plan,
            &self.corpus.expected_check,
        )?;
        row.accept(parsed, evidence);
        Ok(())
    }
}
