use std::{
    collections::BTreeMap,
    env,
    fmt::Write as _,
    fs,
    io::{BufRead, BufReader},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    config::{Backend, Implementation, IrPolicy, Plan, PythonEngine, RuntimeConfig, WasmEngine},
    process::{CommandRecord, Session},
};

pub fn sha256(path: &Path) -> Result<String> {
    let mut digest = Sha256::new();
    let mut reader = BufReader::new(fs::File::open(path)?);
    loop {
        let bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        digest.update(bytes);
        let count = bytes.len();
        reader.consume(count);
    }
    let mut hex = String::with_capacity(64);
    for byte in digest.finalize() {
        write!(hex, "{byte:02x}")?;
    }
    Ok(hex)
}

fn rooted(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() || path.components().count() == 1 {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn executable(root: &Path, path: &Path) -> Result<PathBuf> {
    let path = rooted(root, path);
    let paths: Vec<_> = if path.components().count() == 1 {
        env::var_os("PATH")
            .map(|value| {
                env::split_paths(&value)
                    .map(|dir| root.join(dir).join(&path))
                    .collect()
            })
            .context("PATH is unavailable")?
    } else {
        vec![path.clone()]
    };
    for candidate in paths {
        if fs::metadata(&candidate)
            .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        {
            // Preserve the invocation name: clang++ and toolchain wrappers
            // intentionally dispatch on argv[0].
            return Ok(candidate);
        }
    }
    bail!("executable unavailable: {}", path.display())
}

fn command(root: &Path, path: &Path) -> Command {
    Command::new(rooted(root, path))
}

fn build_command(session: &Session<'_>, cfg: &RuntimeConfig, tool: &Path) -> Command {
    cfg.build_cpus.as_ref().map_or_else(
        || command(session.root(), tool),
        |cpus| {
            let mut cmd = Command::new("taskset");
            cmd.args(["-c", cpus]).arg(rooted(session.root(), tool));
            cmd
        },
    )
}

fn managed_cache(session: &Session<'_>, cfg: &RuntimeConfig) -> Result<PathBuf> {
    ensure!(
        cfg.sccache_client
            .file_name()
            .is_some_and(|name| name.to_string_lossy().contains("sccache")),
        "source builds require an explicit managed sccache client"
    );
    executable(session.root(), &cfg.sccache_client)
}

fn prevent_cache_start(command: &mut Command) {
    command
        .env("SCCACHE_START_SERVER", "0")
        .env("SCCACHE_NO_AUTO_START_SERVER", "1");
}

fn check_cache_diagnostics(stderr: &str) -> Result<()> {
    let lower = stderr.to_ascii_lowercase();
    ensure!(
        ![
            "compiling locally instead",
            "server looks like it shut down unexpectedly",
            "failed to connect to sccache server",
            "sccache: error:"
        ]
        .iter()
        .any(|pattern| lower.contains(pattern)),
        "managed cache reported an infrastructure failure or uncached fallback"
    );
    Ok(())
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    target: Option<CargoTarget>,
    executable: Option<PathBuf>,
}

#[derive(Deserialize)]
struct CargoTarget {
    name: String,
}

fn unique_cargo_artifact(stdout: &str, binary: &str) -> Result<PathBuf> {
    let mut found = None;
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let message: CargoMessage =
            serde_json::from_str(line).context("Cargo JSON artifact stream")?;
        if message.reason == "compiler-artifact"
            && message.target.is_some_and(|target| target.name == binary)
            && let Some(path) = message.executable
        {
            ensure!(
                found.replace(path).is_none(),
                "multiple Cargo artifacts for {binary}"
            );
        }
    }
    found.with_context(|| format!("Cargo did not report executable {binary}"))
}

fn rust_build(
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    toolchain: &str,
    binary: &str,
) -> Result<PathBuf> {
    let wrapper = managed_cache(session, cfg)?;
    let mut build = build_command(session, cfg, &cfg.cargo);
    prevent_cache_start(&mut build);
    build
        .env("RUSTC_WRAPPER", wrapper)
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .arg(format!("+{toolchain}"))
        .args(["build", "--locked", "--release", "--manifest-path"])
        .arg(source.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(session.path("target"))
        .arg("--message-format=json-render-diagnostics");
    let output = session.invoke(build, "build")?;
    check_cache_diagnostics(&output.stderr)?;
    if let Err(error) = output.ensure_success("Cargo build") {
        session.diagnostics.push_str(&output.stdout);
        return Err(error);
    }
    let path = unique_cargo_artifact(&output.stdout, binary)?;
    ensure!(
        path.is_file(),
        "Cargo artifact disappeared: {}",
        path.display()
    );
    Ok(path)
}

fn rust_version(session: &mut Session<'_>, cfg: &RuntimeConfig, toolchain: &str) -> Result<String> {
    let mut cmd = command(session.root(), &cfg.rustc);
    cmd.arg(format!("+{toolchain}")).arg("--version");
    session.version(cmd)
}

fn llvm_flags(session: &mut Session<'_>, config: &Path, args: &[&str]) -> Result<Vec<String>> {
    let mut cmd = Command::new(config);
    cmd.args(args);
    let value = session.checked(cmd, "llvm-config")?;
    shlex::split(&value).context("invalid llvm-config flag quoting")
}

fn library_path(command: &mut Command, directory: &Path) -> Result<()> {
    ensure!(
        directory.is_dir(),
        "library directory unavailable: {}",
        directory.display()
    );
    let mut paths = vec![directory.to_owned()];
    if let Some(inherited) = env::var_os("LD_LIBRARY_PATH") {
        paths.extend(env::split_paths(&inherited));
    }
    command.env("LD_LIBRARY_PATH", env::join_paths(paths)?);
    Ok(())
}

/// Pinned LLVM21 static archive build, preserving target libm symbol binding.
pub fn build_llvm(session: &mut Session<'_>, cfg: &RuntimeConfig, output: &Path) -> Result<()> {
    let config = executable(session.root(), &cfg.llvm_config)?;
    let wrapper = managed_cache(session, cfg)?;
    let mut version = Command::new(&config);
    version.arg("--version");
    let version = session.version(version)?;
    ensure!(version == "21.1.8", "LLVM 21.1.8 required, found {version}");
    let real_config = config.canonicalize()?;
    let bin = real_config
        .parent()
        .context("llvm-config must have a parent directory")?;
    let compiler = executable(session.root(), &bin.join("clang++"))?;
    let linker = executable(session.root(), &bin.join("ld.lld"))?;
    let cflags = llvm_flags(session, &config, &["--cxxflags"])?;
    let ldflags = llvm_flags(session, &config, &["--ldflags"])?;
    let libs = llvm_flags(
        session,
        &config,
        &[
            "--link-static",
            "--libs",
            "orcjit",
            "native",
            "core",
            "support",
        ],
    )?;
    let mut system_libs = llvm_flags(session, &config, &["--system-libs"])?;
    for flag in &mut system_libs {
        if flag.starts_with("/usr/lib/") && flag.ends_with("/libzstd.a") {
            *flag = "-lzstd".into();
        }
    }
    let mut build = build_command(session, cfg, &wrapper);
    prevent_cache_start(&mut build);
    build
        .arg(compiler)
        .args(cflags)
        .args([
            "-O2",
            "-std=c++20",
            "-DNDEBUG",
            "-Wall",
            "-Wextra",
            "-Wpedantic",
        ])
        .arg(format!("-fuse-ld={}", linker.display()))
        .arg(
            session
                .root()
                .join("workloads/hashwx-fixed-v1/implementations/llvm-orc/main.cpp"),
        )
        .arg("-o")
        .arg(output);
    if let Some(directory) = &cfg.llvm_target_libm_dir {
        let directory = session.root().join(directory);
        ensure!(
            directory.join("libm.so").is_file(),
            "target libm.so unavailable"
        );
        build.arg(format!("-L{}", directory.display()));
    }
    if let Some(directory) = &cfg.llvm_link_libdir {
        let directory = session.root().join(directory);
        library_path(&mut build, &directory)?;
        build.arg(format!("-L{}", directory.display()));
    }
    build
        .args(ldflags)
        .arg("-Wl,--as-needed")
        .args(libs)
        .args(system_libs)
        .args(["-pthread", "-ldl", "-lz"]);
    let result = session.invoke(build, "build")?;
    check_cache_diagnostics(&result.stderr)?;
    if let Err(error) = result.ensure_success("LLVM build") {
        session.diagnostics.push_str(&result.stdout);
        return Err(error);
    }
    ensure!(output.is_file(), "LLVM build did not produce output");
    Ok(())
}

fn llvm_command(
    session: &Session<'_>,
    cfg: &RuntimeConfig,
    binary: &Path,
    policy: IrPolicy,
) -> Result<Command> {
    let mut cmd = Command::new(binary);
    cmd.args(["--ir-opt", policy.name()]);
    if let Some(dir) = &cfg.llvm_runtime_libdir {
        library_path(&mut cmd, &session.root().join(dir))?;
    }
    Ok(cmd)
}

fn dotnet_command(session: &Session<'_>, cfg: &RuntimeConfig) -> Command {
    let mut cmd = command(session.root(), &cfg.dotnet);
    dotnet_environment(session, &mut cmd);
    cmd
}

fn dotnet_environment(session: &Session<'_>, cmd: &mut Command) {
    cmd.env("DOTNET_CLI_HOME", session.path("dotnet-home"))
        .env("DOTNET_NOLOGO", "1")
        .env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
}

fn dotnet_build(
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    project: &str,
    output: &Path,
) -> Result<PathBuf> {
    let mut build = build_command(session, cfg, &cfg.dotnet);
    dotnet_environment(session, &mut build);
    build
        .arg("publish")
        .arg(source.join(format!("{project}.csproj")))
        .arg("--artifacts-path")
        .arg(session.path("artifacts"))
        .args(["--disable-build-servers", "-c", "Release", "-o"])
        .arg(output)
        .args(["--nologo", "-v", "quiet"]);
    session.checked(build, "build")?;
    let assembly = output.join(format!("{project}.dll"));
    ensure!(
        assembly.is_file(),
        "missing published assembly: {}",
        assembly.display()
    );
    Ok(assembly)
}

const INLINE_EXPORT: &str = "java.base/jdk.internal.vm.annotation=ALL-UNNAMED";

fn java_build(
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    class: &str,
    force_inline: bool,
) -> Result<Command> {
    let mut javac = build_command(session, cfg, &cfg.javac);
    if let Some(release) = cfg.javac_release {
        javac.arg("--release").arg(release.to_string());
    }
    if force_inline {
        javac.args(["--add-exports", INLINE_EXPORT]);
    }
    javac
        .arg("-d")
        .arg(session.scratch())
        .arg(source.join(format!("{class}.java")));
    session.checked(javac, "build")?;
    let mut java = command(session.root(), &cfg.java);
    if force_inline {
        java.args(["--add-exports", INLINE_EXPORT]);
    }
    java.arg("-cp").arg(session.scratch()).arg(class);
    Ok(java)
}

fn uv_python(session: &Session<'_>, cfg: &RuntimeConfig) -> Command {
    let mut cmd = command(session.root(), &cfg.uv);
    cmd.args(["run", "--no-project", "--python", "3"]);
    cmd
}

fn wasmtime_python(session: &Session<'_>, cfg: &RuntimeConfig) -> Command {
    cfg.wasmtime_python.as_ref().map_or_else(
        || {
            let mut cmd = uv_python(session, cfg);
            cmd.args(["--with", "wasmtime==50.0.1", "python"]);
            cmd
        },
        |python| command(session.root(), python),
    )
}

fn emit_wasm(
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    output: &Path,
) -> Result<PathBuf> {
    let path = output.join("hashwx-fixed.wasm");
    let mut cmd = uv_python(session, cfg);
    cmd.arg("python")
        .arg(source.join("emit_wasm.py"))
        .arg("--output")
        .arg(&path);
    session.checked(cmd, "build")?;
    ensure!(path.is_file(), "Wasm emitter did not produce module");
    Ok(path)
}

/// Artifacts are shared only within one invocation, never persisted in a
/// second compiler cache. Identity is the source manifest/translation unit,
/// and all runtime/build settings are immutable for this invocation.
pub struct Artifacts {
    root: tempfile::TempDir,
    built: BTreeMap<PathBuf, Artifact>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildProvenance {
    reused: bool,
    commands: Vec<CommandRecord>,
}

#[derive(Clone)]
struct Artifact {
    path: PathBuf,
    sha256: String,
    build: Option<BuildProvenance>,
}

impl Artifact {
    fn external(path: PathBuf) -> Result<Self> {
        Ok(Self {
            sha256: sha256(&path)?,
            path,
            build: None,
        })
    }
}

impl Artifacts {
    pub fn new() -> Result<Self> {
        Ok(Self {
            root: tempfile::Builder::new()
                .prefix("museum-artifacts-")
                .tempdir()?,
            built: BTreeMap::new(),
        })
    }

    fn acquire(
        &mut self,
        key: PathBuf,
        session: &mut Session<'_>,
        build: impl FnOnce(&mut Session<'_>, &Path) -> Result<PathBuf>,
    ) -> Result<Artifact> {
        if let Some(cached) = self.built.get(&key) {
            let mut artifact = cached.clone();
            if let Some(build) = &mut artifact.build {
                build.reused = true;
            }
            return Ok(artifact);
        }
        let directory = tempfile::Builder::new()
            .prefix("build-")
            .tempdir_in(self.root.path())?;
        let start = session.records.len();
        let path = build(session, directory.path())?;
        ensure!(
            path.starts_with(directory.path()),
            "build output escaped owned artifact directory"
        );
        let artifact = Artifact {
            sha256: sha256(&path)?,
            path,
            build: Some(BuildProvenance {
                reused: false,
                commands: session.records[start..].to_vec(),
            }),
        };
        // The invocation-level root now owns cleanup for the reused artifact.
        let _directory = directory.keep();
        self.built.insert(key, artifact.clone());
        Ok(artifact)
    }

    fn cargo(
        &mut self,
        session: &mut Session<'_>,
        cfg: &RuntimeConfig,
        source: &Path,
        toolchain: &str,
        binary: &str,
    ) -> Result<Artifact> {
        self.acquire(source.join("Cargo.toml"), session, |session, directory| {
            let executable = rust_build(session, cfg, source, toolchain, binary)?;
            let output = directory.join(binary);
            // Cargo's ephemeral intermediates may live in session scratch.
            // Retain exactly its reported final executable, not build trees.
            fs::copy(executable, &output)?;
            Ok(output)
        })
    }
}

pub struct Prepared {
    pub command: Command,
    pub version: String,
    pub binary_sha256: Option<String>,
    pub build_provenance: Option<BuildProvenance>,
}

impl Prepared {
    fn new(command: Command, version: String, artifact: Option<Artifact>) -> Self {
        let (binary_sha256, build_provenance) = artifact.map_or((None, None), |artifact| {
            (Some(artifact.sha256), artifact.build)
        });
        Self {
            command,
            version,
            binary_sha256,
            build_provenance,
        }
    }
}

fn prepare_rust(
    assets: &mut Artifacts,
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    challenge: bool,
) -> Result<Prepared> {
    let (toolchain, binary) = if challenge {
        ("nightly-2026-11-03", "museum-challenge-sha256-reference")
    } else {
        ("1.99.0", "hashwx-zipp-raw")
    };
    let (artifact, version) =
        if let Some(prebuilt) = cfg.rust_challenge.as_ref().filter(|_| challenge) {
            (
                Artifact::external(executable(session.root(), &prebuilt.binary)?)?,
                prebuilt.rustc_version.clone(),
            )
        } else {
            let version = rust_version(session, cfg, toolchain)?;
            (
                assets.cargo(session, cfg, source, toolchain, binary)?,
                version,
            )
        };
    Ok(Prepared::new(
        Command::new(&artifact.path),
        version,
        Some(artifact),
    ))
}

fn prepare_cranelift(
    assets: &mut Artifacts,
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    corpus: &Path,
    batch: bool,
) -> Result<Prepared> {
    let artifact = if let Some(binary) = &cfg.cranelift_binary {
        Artifact::external(executable(session.root(), binary)?)?
    } else {
        assets.cargo(
            session,
            cfg,
            source,
            "1.99.0",
            "museum-hashwx-cranelift-direct",
        )?
    };
    let mut probe = Command::new(&artifact.path);
    probe.arg("--version");
    let version = session.version(probe)?;
    ensure!(
        version.contains("Cranelift 0.137.1"),
        "unexpected Cranelift version: {version}"
    );
    let mut cmd = Command::new(&artifact.path);
    cmd.arg("--corpus").arg(corpus);
    if batch {
        cmd.arg("--batch");
    }
    Ok(Prepared::new(cmd, version, Some(artifact)))
}

fn prepare_llvm(
    assets: &mut Artifacts,
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    corpus: &Path,
    policy: IrPolicy,
) -> Result<Prepared> {
    let artifact = if let Some(binary) = &cfg.llvm_binary {
        Artifact::external(executable(session.root(), binary)?)?
    } else {
        assets.acquire(source.join("main.cpp"), session, |session, directory| {
            let path = directory.join("llvm-orc-hashwx");
            build_llvm(session, cfg, &path)?;
            Ok(path)
        })?
    };
    let mut probe = llvm_command(session, cfg, &artifact.path, policy)?;
    probe.arg("--version");
    let version = session.version(probe)?;
    let mut cmd = llvm_command(session, cfg, &artifact.path, policy)?;
    cmd.arg("--corpus").arg(corpus);
    Ok(Prepared::new(cmd, version, Some(artifact)))
}

fn prepare_dotnet(
    assets: &mut Artifacts,
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    project: &str,
    challenge: Option<crate::config::ChallengeMode>,
) -> Result<Prepared> {
    let mut probe = dotnet_command(session, cfg);
    probe.arg("--version");
    let version = session.version(probe)?;
    let artifact = assets.acquire(
        source.join(format!("{project}.csproj")),
        session,
        |session, directory| dotnet_build(session, cfg, source, project, directory),
    )?;
    let mut cmd = dotnet_command(session, cfg);
    cmd.arg(&artifact.path);
    if let Some(mode) = challenge {
        cmd.args(["--mode", mode.name()]);
    }
    Ok(Prepared::new(cmd, version, Some(artifact)))
}

fn prepare_python(
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    engine: PythonEngine,
) -> Result<Prepared> {
    let (path, mode) = match engine {
        PythonEngine::Cpython => (&cfg.cpython, "cpython"),
        PythonEngine::Pypy => (&cfg.pypy, "pypy"),
    };
    let mut probe = command(session.root(), path);
    probe.arg("--version");
    let version = session.version(probe)?;
    let mut cmd = command(session.root(), path);
    if matches!(engine, PythonEngine::Cpython) {
        cmd.env("PYTHON_JIT", "1");
    }
    cmd.arg("-u")
        .arg(source.join("bench.py"))
        .args(["--engine", mode]);
    Ok(Prepared::new(cmd, version, None))
}

fn prepare_wasm(
    assets: &mut Artifacts,
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    source: &Path,
    engine: WasmEngine,
) -> Result<Prepared> {
    let module = assets.acquire(
        source.join("emit_wasm.py"),
        session,
        |session, directory| emit_wasm(session, cfg, source, directory),
    )?;
    let (version, mut cmd) = match engine {
        WasmEngine::V8 => {
            let mut probe = command(session.root(), &cfg.node);
            probe.args(["-p", "process.version + ' / V8 ' + process.versions.v8"]);
            let version = session.version(probe)?;
            let mut cmd = command(session.root(), &cfg.node);
            cmd.arg(source.join("v8.mjs"));
            (version, cmd)
        }
        WasmEngine::Wasmtime => {
            let mut probe = wasmtime_python(session, cfg);
            probe.args(["-c", "import importlib.metadata; print('wasmtime ' + importlib.metadata.version('wasmtime'))"]);
            let version = session.version(probe)?;
            let mut cmd = wasmtime_python(session, cfg);
            cmd.arg(source.join("wasmtime_host.py"));
            (version, cmd)
        }
    };
    cmd.arg(&module.path);
    Ok(Prepared::new(cmd, version, Some(module)))
}

pub fn prepare(
    assets: &mut Artifacts,
    session: &mut Session<'_>,
    cfg: &RuntimeConfig,
    implementation: &Implementation,
    corpus: &Path,
    plan: Plan,
) -> Result<Prepared> {
    let source = session.root().join(&implementation.source);
    let mut prepared = match &implementation.backend {
        Backend::RustDynasm => prepare_rust(assets, session, cfg, &source, false),
        Backend::RustChallenge => prepare_rust(assets, session, cfg, &source, true),
        Backend::Dotnet { project, challenge } => {
            prepare_dotnet(assets, session, cfg, &source, project, *challenge)
        }
        Backend::Java {
            class,
            force_inline,
        } => {
            let mut probe = command(session.root(), &cfg.java);
            probe.arg("-version");
            let version = session.version(probe)?;
            let cmd = java_build(session, cfg, &source, class, *force_inline)?;
            Ok(Prepared::new(cmd, version, None))
        }
        Backend::Python { engine } => prepare_python(session, cfg, &source, *engine),
        Backend::Rpython => {
            let path = executable(
                session.root(),
                cfg.rpython_binary
                    .as_deref()
                    .context("rpython_binary is required")?,
            )?;
            let mut probe = Command::new(&path);
            probe.arg("--version");
            let version = session.version(probe)?;
            Ok(Prepared::new(
                Command::new(&path),
                version,
                Some(Artifact::external(path)?),
            ))
        }
        Backend::Wasm { engine } => prepare_wasm(assets, session, cfg, &source, *engine),
        Backend::Cranelift { batch } => {
            prepare_cranelift(assets, session, cfg, &source, corpus, *batch)
        }
        Backend::Llvm { policy } => prepare_llvm(assets, session, cfg, &source, corpus, *policy),
    }?;
    prepared.command.args(plan.dimensions());
    Ok(prepared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hashes_empty_and_multi_buffer_files() -> Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        assert_eq!(
            sha256(file.path())?,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        fs::write(file.path(), vec![b'a'; 1_000_000])?;
        assert_eq!(
            sha256(file.path())?,
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
        Ok(())
    }

    #[test]
    fn cargo_artifacts_are_selected_by_identity() -> Result<()> {
        let stream = r#"{"reason":"compiler-artifact","target":{"name":"dep"},"executable":null}
{"reason":"compiler-artifact","target":{"name":"benchmark"},"executable":"/tmp/exe"}
{"reason":"build-finished","success":true}"#;
        assert_eq!(
            unique_cargo_artifact(stream, "benchmark")?,
            Path::new("/tmp/exe")
        );
        assert!(unique_cargo_artifact(stream, "wrong").is_err());
        assert!(unique_cargo_artifact(&format!("{stream}\n{stream}"), "benchmark").is_err());
        assert!(check_cache_diagnostics("sccache: warning: compiling locally instead").is_err());
        Ok(())
    }
}
