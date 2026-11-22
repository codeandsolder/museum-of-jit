mod backend;
mod config;
mod process;
mod protocol;
mod result;

use std::{
    fs,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use clap::Parser;

use config::{Action, Catalog, Cli, Plan, RuntimeConfig, Workload};
use process::Session;
use result::{Corpus, Host, Revision, RunContext};

fn revision(root: &Path, cancelled: &AtomicBool) -> Result<Revision> {
    let mut session = Session::new(root, Duration::from_secs(20), cancelled)?;
    let mut git = Command::new("git");
    git.args(["rev-parse", "HEAD"]);
    let commit = session.checked(git, "revision")?.trim().to_owned();
    let mut status = Command::new("git");
    status.args(["status", "--porcelain"]);
    let dirty = !session.checked(status, "revision")?.is_empty();
    Ok(Revision { commit, dirty })
}

fn write_results(path: Option<&Path>, root: &Path, rows: &[result::Row<'_>]) -> Result<()> {
    if let Some(path) = path {
        let path = root.join(path);
        let parent = path
            .parent()
            .context("output file needs a parent directory")?;
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut file, rows)?;
        writeln!(file)?;
        file.persist(&path)?;
        eprintln!("{}", path.display());
    } else {
        let mut output = io::stdout().lock();
        serde_json::to_writer_pretty(&mut output, rows)?;
        writeln!(output)?;
    }
    Ok(())
}

struct RunArgs {
    workload: Workload,
    implementations: Vec<String>,
    plan: Plan,
    output: Option<PathBuf>,
}

fn run(
    root: &Path,
    cfg: &RuntimeConfig,
    args: &RunArgs,
    cancelled: &AtomicBool,
) -> Result<ExitCode> {
    let timeout = args.plan.validate(args.workload)?;
    let catalog = Catalog::load(root, args.workload)?;
    let implementations = catalog.select(&args.implementations)?;
    let host = Host::read()?;
    let corpus = Corpus::load(root, args.workload)?;
    let revision = revision(root, cancelled)?;
    let context = RunContext {
        root,
        config: cfg,
        host: &host,
        corpus: &corpus,
        revision: &revision,
        plan: args.plan,
        timeout,
        cancelled,
    };
    let mut assets = backend::Artifacts::new()?;
    let mut rows = Vec::new();
    for implementation in implementations {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        rows.push(context.run(&mut assets, implementation)?);
    }
    write_results(args.output.as_deref(), root, &rows)?;
    // Every attempted exhibit is preserved in the raw output, including
    // intentional negative exhibits, timeouts and actual backend failures.
    Ok(if cancelled.load(Ordering::Relaxed) {
        ExitCode::from(130)
    } else if rows.iter().all(result::Row::succeeded) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn build_llvm(
    root: &Path,
    cfg: &RuntimeConfig,
    output: &Path,
    timeout: u64,
    cancelled: &AtomicBool,
) -> Result<()> {
    ensure!(timeout > 0, "timeout must be positive");
    let output = root.join(output);
    let parent = output.parent().context("output must have a parent")?;
    fs::create_dir_all(parent)?;
    ensure!(
        !parent.canonicalize()?.starts_with(root),
        "build output must be outside the repository"
    );
    let mut session = Session::new(root, Duration::from_secs(timeout), cancelled)?;
    let artifact = session.path("llvm-orc-hashwx");
    let result = backend::build_llvm(&mut session, cfg, &artifact);
    eprint!("{}", session.diagnostics);
    result?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    io::copy(&mut fs::File::open(&artifact)?, &mut file)?;
    file.as_file()
        .set_permissions(fs::Permissions::from_mode(0o755))?;
    file.persist(&output)?;
    eprintln!("{} sha256={}", output.display(), backend::sha256(&output)?);
    Ok(())
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let root = cli.root.canonicalize()?;
    let config = RuntimeConfig::load(cli.config.as_deref())?;
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancelled);
    ctrlc::set_handler(move || signal.store(true, Ordering::Relaxed))?;
    nix::sys::prctl::set_child_subreaper(true)?;
    match cli.action {
        Action::List { workload } => {
            let catalog = Catalog::load(&root, workload)?;
            for item in catalog.implementation {
                println!("{:24} {:32?} {}", item.id, item.mode, item.runtime);
            }
            Ok(ExitCode::SUCCESS)
        }
        Action::Run {
            workload,
            implementations,
            plan,
            output,
        } => run(
            &root,
            &config,
            &RunArgs {
                workload,
                implementations,
                plan,
                output,
            },
            &cancelled,
        ),
        Action::BuildLlvm { output, timeout } => {
            build_llvm(&root, &config, &output, timeout, &cancelled)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}
