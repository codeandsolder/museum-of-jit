use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    num::NonZeroU32,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Parser)]
#[command(version, about = "Run the Museum's reproducible benchmark exhibits")]
pub(crate) struct Cli {
    /// Repository root (defaults to the current directory).
    #[arg(long, global = true, default_value = ".")]
    pub root: PathBuf,
    /// Explicit runtime selection file. No implicit configuration discovery.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Action {
    List {
        #[arg(long, value_enum, default_value_t = Workload::HashwxFixedV1)]
        workload: Workload,
    },
    Run {
        #[arg(long, value_enum, default_value_t = Workload::HashwxFixedV1)]
        workload: Workload,
        /// Implementation IDs; omit to run all.
        #[arg(long = "impl")]
        implementations: Vec<String>,
        #[command(flatten)]
        plan: Plan,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Build a standalone artifact with the pinned LLVM toolchain.
    BuildLlvm {
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1200)]
        timeout: u64,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Workload {
    HashwxFixedV1,
    ChallengeSha256V1,
}

impl Workload {
    pub const fn id(self) -> &'static str {
        match self {
            Self::HashwxFixedV1 => "hashwx-fixed-v1",
            Self::ChallengeSha256V1 => "challenge-sha256-v1",
        }
    }

    const fn max_count(self) -> u32 {
        match self {
            Self::HashwxFixedV1 => i32::MAX as u32,
            Self::ChallengeSha256V1 => 160_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, clap::Args)]
pub(crate) struct Plan {
    #[arg(long, default_value = "30000", value_parser = decimal_count)]
    pub warmup: u32,
    #[arg(long, default_value = "300000", value_parser = positive_count)]
    pub iterations: NonZeroU32,
    #[arg(long, default_value = "7", value_parser = positive_count)]
    pub samples: NonZeroU32,
    /// Whole-run deadline including compilation, in seconds.
    #[arg(long, default_value_t = 120.0)]
    pub timeout: f64,
}

fn decimal_count(value: &str) -> Result<u32, String> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("expected an unsigned decimal integer".into());
    }
    value.parse().map_err(|_| "count exceeds u32 range".into())
}

fn positive_count(value: &str) -> Result<NonZeroU32, String> {
    NonZeroU32::new(decimal_count(value)?).ok_or_else(|| "count must be positive".into())
}

impl Plan {
    pub fn validate(self, workload: Workload) -> Result<Duration> {
        ensure!(
            [self.warmup, self.iterations.get(), self.samples.get()]
                .into_iter()
                .all(|n| n <= workload.max_count()),
            "counts exceed workload limit {}",
            workload.max_count()
        );
        let duration = Duration::try_from_secs_f64(self.timeout).context("invalid timeout")?;
        ensure!(!duration.is_zero(), "timeout must be positive");
        Ok(duration)
    }

    pub fn dimensions(self) -> [String; 3] {
        [
            self.warmup.to_string(),
            self.iterations.to_string(),
            self.samples.to_string(),
        ]
    }

    pub fn attestation(self) -> String {
        format!("BENCHMARK_DIMENSIONS v1 {}", self.dimensions().join(" "))
    }
}

/// Host-specific paths, not benchmark settings. Paths with separators are
/// interpreted relative to the repository root. Bare executable names use PATH.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct RuntimeConfig {
    pub cargo: PathBuf,
    pub rustc: PathBuf,
    pub sccache_client: PathBuf,
    pub dotnet: PathBuf,
    pub java: PathBuf,
    pub javac: PathBuf,
    pub javac_release: Option<NonZeroU32>,
    pub cpython: PathBuf,
    pub pypy: PathBuf,
    pub node: PathBuf,
    pub uv: PathBuf,
    pub wasmtime_python: Option<PathBuf>,
    pub rpython_binary: Option<PathBuf>,
    pub cranelift_binary: Option<PathBuf>,
    pub rust_challenge_binary: Option<PathBuf>,
    pub rust_challenge_rustc_version: Option<String>,
    pub llvm_binary: Option<PathBuf>,
    pub llvm_config: PathBuf,
    pub llvm_link_libdir: Option<PathBuf>,
    pub llvm_target_libm_dir: Option<PathBuf>,
    pub llvm_runtime_libdir: Option<PathBuf>,
    pub build_cpus: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            cargo: "cargo".into(),
            rustc: "rustc".into(),
            sccache_client: "/usr/local/libexec/sccache-client".into(),
            dotnet: "dotnet".into(),
            java: "java".into(),
            javac: "javac".into(),
            javac_release: None,
            cpython: "python3".into(),
            pypy: "pypy3".into(),
            node: "node".into(),
            uv: "uv".into(),
            wasmtime_python: None,
            rpython_binary: None,
            cranelift_binary: None,
            rust_challenge_binary: None,
            rust_challenge_rustc_version: None,
            llvm_binary: None,
            llvm_config: "llvm-config".into(),
            llvm_link_libdir: None,
            llvm_target_libm_dir: None,
            llvm_runtime_libdir: None,
            build_cpus: None,
        }
    }
}

impl RuntimeConfig {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        path.map_or_else(
            || Ok(Self::default()),
            |p| {
                let text =
                    fs::read_to_string(p).with_context(|| format!("read {}", p.display()))?;
                toml::from_str(&text).with_context(|| format!("parse {}", p.display()))
            },
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalog {
    pub implementation: Vec<Implementation>,
}

impl Catalog {
    pub fn load(root: &Path, workload: Workload) -> Result<Self> {
        let path = root
            .join("workloads")
            .join(workload.id())
            .join("implementations.toml");
        let catalog: Self = toml::from_str(&fs::read_to_string(path)?)?;
        let mut ids = std::collections::BTreeSet::new();
        for item in &catalog.implementation {
            ensure!(ids.insert(&item.id), "duplicate implementation {}", item.id);
            ensure!(
                root.join(&item.source).is_dir(),
                "missing source {}",
                item.source.display()
            );
        }
        Ok(catalog)
    }

    pub fn select(&self, requested: &[String]) -> Result<Vec<&Implementation>> {
        if requested.is_empty() {
            return Ok(self.implementation.iter().collect());
        }
        let mut seen = std::collections::BTreeSet::new();
        requested
            .iter()
            .map(|id| {
                ensure!(seen.insert(id), "duplicate selection {id}");
                self.implementation
                    .iter()
                    .find(|item| &item.id == id)
                    .with_context(|| format!("unknown implementation {id}"))
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Implementation {
    pub id: String,
    pub mode: ImplementationMode,
    pub runtime: String,
    pub source: PathBuf,
    pub backend: Backend,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ImplementationMode {
    Idiomatic,
    GeneratedSourceOrBytecode,
    DirectIrOrMachineCode,
    MetaJitInterpreter,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ChallengeMode {
    Portable,
    Optimized,
    MatchOptimized,
    Match,
    Scalar,
    Bcl,
}

impl ChallengeMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Portable => "portable",
            Self::Optimized => "optimized",
            Self::MatchOptimized => "match-optimized",
            Self::Match => "match",
            Self::Scalar => "scalar",
            Self::Bcl => "bcl",
        }
    }

    pub const fn hit_only(self) -> bool {
        matches!(self, Self::Match | Self::MatchOptimized)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(crate) enum IrPolicy {
    O0,
    O1,
    O2,
    O3,
}

impl IrPolicy {
    pub const fn name(self) -> &'static str {
        match self {
            Self::O0 => "O0",
            Self::O1 => "O1",
            Self::O2 => "O2",
            Self::O3 => "O3",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PythonEngine {
    Cpython,
    Pypy,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WasmEngine {
    V8,
    Wasmtime,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Backend {
    RustDynasm,
    RustChallenge,
    Dotnet {
        project: String,
        challenge: Option<ChallengeMode>,
    },
    Java {
        class: String,
        force_inline: bool,
    },
    Python {
        engine: PythonEngine,
    },
    Rpython,
    Wasm {
        engine: WasmEngine,
    },
    Cranelift {
        batch: bool,
    },
    Llvm {
        policy: IrPolicy,
    },
}
