use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Seek, SeekFrom},
    os::unix::process::{CommandExt, ExitStatusExt},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use nix::{
    errno::Errno,
    sys::{
        signal::{Signal, killpg},
        wait::{Id, WaitPidFlag, WaitStatus, waitid, waitpid},
    },
    unistd::Pid,
};
use serde::Serialize;
use tempfile::TempDir;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Keep the process group leader unreaped until its descendants have been
/// signalled, so its numeric process-group ID cannot be recycled mid-cleanup.
struct GroupChild {
    child: Child,
    group: Pid,
    finished: bool,
}

impl GroupChild {
    fn spawn(command: &mut Command) -> Result<Self> {
        let child = command.process_group(0).spawn()?;
        let group = Pid::from_raw(i32::try_from(child.id())?);
        Ok(Self {
            child,
            group,
            finished: false,
        })
    }

    fn terminate(&mut self) -> Result<ExitStatus> {
        match killpg(self.group, Signal::SIGKILL) {
            Ok(()) | Err(Errno::ESRCH) => {}
            Err(error) => return Err(error.into()),
        }
        let status = self.child.wait()?;
        // The harness is a Linux subreaper. Reap adopted group members,
        // including grandchildren that were left behind by a failed tool.
        loop {
            match waitpid(Pid::from_raw(-self.group.as_raw()), None) {
                Err(Errno::ECHILD) => break,
                Err(Errno::EINTR) | Ok(_) => {}
                Err(error) => return Err(error.into()),
            }
        }
        self.finished = true;
        Ok(status)
    }

    fn wait(&mut self, deadline: Instant, cancelled: &AtomicBool) -> Result<(ExitStatus, bool)> {
        let flags = WaitPidFlag::WEXITED | WaitPidFlag::WNOHANG | WaitPidFlag::WNOWAIT;
        loop {
            if cancelled.load(Ordering::Relaxed) || Instant::now() >= deadline {
                return Ok((self.terminate()?, true));
            }
            match waitid(Id::Pid(self.group), flags) {
                Ok(WaitStatus::StillAlive) | Err(Errno::EINTR) => {
                    thread::sleep(
                        POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())),
                    );
                }
                Ok(_) => return Ok((self.terminate()?, false)),
                Err(error) => return Err(error.into()),
            }
        }
    }
}

impl Drop for GroupChild {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.terminate();
        }
    }
}

fn read_output(file: &mut File) -> Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn utf8(value: &OsStr) -> Result<String> {
    value
        .to_str()
        .map(str::to_owned)
        .context("non-UTF-8 command provenance")
}

#[derive(Debug, Serialize)]
pub(crate) struct CommandRecord {
    pub argv: Vec<String>,
    pub environment: std::collections::BTreeMap<String, Option<String>>,
    pub stage: &'static str,
    pub exit_code: Option<i32>,
    pub elapsed_seconds: f64,
}

impl CommandRecord {
    fn new(command: &Command, stage: &'static str) -> Result<Self> {
        let argv = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(utf8)
            .collect::<Result<_>>()?;
        let environment = command
            .get_envs()
            .map(|(key, value)| Ok((utf8(key)?, value.map(utf8).transpose()?)))
            .collect::<Result<_>>()?;
        Ok(Self {
            argv,
            environment,
            stage,
            exit_code: None,
            elapsed_seconds: 0.0,
        })
    }
}

pub(crate) struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// One benchmark owns a scratch directory, an absolute deadline, and all
/// external invocations. No subprocess inherits a writable output pipe.
pub(crate) struct Session<'a> {
    root: &'a Path,
    scratch: TempDir,
    deadline: Instant,
    cancelled: &'a AtomicBool,
    pub records: Vec<CommandRecord>,
    pub diagnostics: String,
    pub timed_out: bool,
    pub last_exit: Option<i32>,
}

impl<'a> Session<'a> {
    pub fn new(root: &'a Path, timeout: Duration, cancelled: &'a AtomicBool) -> Result<Self> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .context("deadline overflow")?;
        Ok(Self {
            root,
            scratch: tempfile::Builder::new().prefix("museum-run-").tempdir()?,
            deadline,
            cancelled,
            records: Vec::new(),
            diagnostics: String::new(),
            timed_out: false,
            last_exit: None,
        })
    }

    pub fn root(&self) -> &Path {
        self.root
    }
    pub fn scratch(&self) -> &Path {
        self.scratch.path()
    }
    pub fn path(&self, name: &str) -> PathBuf {
        self.scratch().join(name)
    }

    pub fn invoke(&mut self, mut command: Command, stage: &'static str) -> Result<Output> {
        if self.cancelled.load(Ordering::Relaxed) || Instant::now() >= self.deadline {
            self.timed_out = true;
            bail!("benchmark deadline or cancellation before {stage}");
        }
        command
            .current_dir(self.root)
            .env("TMPDIR", self.scratch.path())
            .stdin(Stdio::null());
        let mut record = CommandRecord::new(&command, stage)?;
        let mut stdout = tempfile::tempfile_in(self.scratch.path())?;
        let mut stderr = tempfile::tempfile_in(self.scratch.path())?;
        command
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?);
        let started = Instant::now();
        let mut child =
            GroupChild::spawn(&mut command).with_context(|| format!("launch {:?}", record.argv))?;
        let (status, timed_out) = child.wait(self.deadline, self.cancelled)?;
        let code = status
            .code()
            .or_else(|| status.signal().map(|signal| -signal))
            .context("unknown command termination")?;
        record.exit_code = (!timed_out).then_some(code);
        record.elapsed_seconds = started.elapsed().as_secs_f64();
        self.last_exit = record.exit_code;
        self.records.push(record);
        let output = Output {
            code,
            stdout: read_output(&mut stdout)?,
            stderr: read_output(&mut stderr)?,
        };
        self.diagnostics.push_str(&output.stderr);
        if timed_out {
            self.timed_out = true;
            self.diagnostics.push_str(&output.stdout);
            bail!("{stage} exceeded deadline or was cancelled");
        }
        Ok(output)
    }

    pub fn checked(&mut self, command: Command, stage: &'static str) -> Result<String> {
        let output = self.invoke(command, stage)?;
        if output.code != 0 {
            self.diagnostics.push_str(&output.stdout);
            bail!("{stage} failed with status {}", output.code);
        }
        if stage == "build" {
            self.diagnostics.push_str(&output.stdout);
        }
        Ok(output.stdout)
    }

    pub fn version(&mut self, command: Command) -> Result<String> {
        let output = self.invoke(command, "version")?;
        ensure!(
            output.code == 0,
            "version probe failed with status {}",
            output.code
        );
        let version = format!("{}{}", output.stdout, output.stderr);
        ensure!(!version.trim().is_empty(), "empty runtime version");
        Ok(version.trim().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_kills_group_and_reaps_children() -> Result<()> {
        nix::sys::prctl::set_child_subreaper(true)?;
        let root = tempfile::tempdir()?;
        let cancelled = AtomicBool::new(false);
        let mut session = Session::new(root.path(), Duration::from_millis(500), &cancelled)?;
        let scratch = session.scratch().to_owned();
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 60 & printf '%s\n' \"$!\"; wait"]);
        assert!(session.invoke(command, "benchmark").is_err());
        assert!(session.timed_out);
        let pid: i32 = session.diagnostics.trim().parse()?;
        assert!(!Path::new("/proc").join(pid.to_string()).exists());
        drop(session);
        assert!(!scratch.exists());
        Ok(())
    }

    #[test]
    fn normal_exit_cleans_leftover_child() -> Result<()> {
        nix::sys::prctl::set_child_subreaper(true)?;
        let root = tempfile::tempdir()?;
        let cancelled = AtomicBool::new(false);
        let mut session = Session::new(root.path(), Duration::from_secs(5), &cancelled)?;
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 60 & printf '%s\n' \"$!\""]);
        let output = session.checked(command, "benchmark")?;
        let pid: i32 = output.trim().parse()?;
        assert!(!Path::new("/proc").join(pid.to_string()).exists());
        Ok(())
    }
}
