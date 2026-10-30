//! Bounded process-tree ownership for real CLI test fixtures.

#![allow(
    dead_code,
    reason = "Each test target uses a subset of the fixture operations"
)]

use std::io;
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, ChildStdout, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use process_wrap::std::{ChildWrapper, CommandWrap};

#[path = "process/capture.rs"]
mod capture;
use capture::Capture;

#[cfg(target_os = "linux")]
#[path = "process/descendants.rs"]
mod descendants;

pub struct Process {
    child: Box<dyn ChildWrapper>,
    deadline: Instant,
    status: Option<ExitStatus>,
    terminated: bool,
    stdout: Capture,
    stderr: Capture,
    line_cursor: usize,
    #[cfg(target_os = "linux")]
    descendants: Option<descendants::Tracker>,
}

impl Process {
    pub fn spawn(command: Command, timeout: Duration) -> io::Result<Self> {
        Self::spawn_inner(command, timeout, true)
    }

    /// Give the caller stdout for an interactive protocol reader; still capture stderr.
    pub fn spawn_raw(command: Command, timeout: Duration) -> io::Result<Self> {
        Self::spawn_inner(command, timeout, false)
    }

    fn spawn_inner(
        mut command: Command,
        timeout: Duration,
        capture_stdout: bool,
    ) -> io::Result<Self> {
        let deadline = Instant::now() + timeout;
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(process_wrap::std::ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(process_wrap::std::JobObject);
        let mut process = Self {
            child: command.spawn()?,
            deadline,
            status: None,
            terminated: false,
            stdout: Capture::default(),
            stderr: Capture::default(),
            line_cursor: 0,
            #[cfg(target_os = "linux")]
            descendants: None,
        };
        #[cfg(target_os = "linux")]
        {
            process.descendants = Some(descendants::Tracker::new(process.id())?);
        }
        if capture_stdout {
            let pipe = process
                .child
                .stdout()
                .take()
                .ok_or_else(|| io::Error::other("fixture stdout"))?;
            #[cfg(unix)]
            capture::nonblocking(&pipe)?;
            process.stdout = Capture::start(pipe)?;
        }
        let pipe = process
            .child
            .stderr()
            .take()
            .ok_or_else(|| io::Error::other("fixture stderr"))?;
        #[cfg(unix)]
        capture::nonblocking(&pipe)?;
        process.stderr = Capture::start(pipe)?;
        Ok(process)
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin().take()
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout().take()
    }

    /// Narrow a phase's remaining time without extending the overall deadline.
    pub fn limit_remaining_time(&mut self, limit: Duration) {
        self.deadline = self.deadline.min(Instant::now() + limit);
    }

    fn contextual(&self, kind: io::ErrorKind, message: &str) -> io::Error {
        io::Error::new(
            kind,
            format!(
                "{message}\nstdout:\n{}\nstderr:\n{}",
                self.stdout.text(),
                self.stderr.text()
            ),
        )
    }

    pub fn diagnostics(&self) -> String {
        format!(
            "stdout:\n{}\nstderr:\n{}",
            self.stdout.text(),
            self.stderr.text()
        )
    }

    pub fn wait_for_stdout_line(&mut self, prefix: &str) -> io::Result<String> {
        loop {
            {
                let bytes = self
                    .stdout
                    .bytes
                    .lock()
                    .map_err(|_| io::Error::other("fixture stdout lock"))?;
                while let Some(end) = bytes[self.line_cursor..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                {
                    let end = self.line_cursor + end + 1;
                    let line = String::from_utf8_lossy(&bytes[self.line_cursor..end]).into_owned();
                    self.line_cursor = end;
                    if line.starts_with(prefix) {
                        drop(bytes);
                        // A real readiness receipt is an ownership checkpoint:
                        // register children before the caller releases its barrier.
                        #[cfg(target_os = "linux")]
                        if let Some(descendants) = self.descendants.as_mut() {
                            descendants.sample(true)?;
                        }
                        return Ok(line);
                    }
                }
            }
            if self.leader_exited()? {
                // A descendant may still hold a Windows pipe open. Terminate
                // owned work before joining blocking readers on this failure path.
                let cleanup = self.terminate();
                // A reader may have published the readiness line just before exit.
                let bytes = self
                    .stdout
                    .bytes
                    .lock()
                    .map_err(|_| io::Error::other("fixture stdout lock"))?;
                for line in bytes[self.line_cursor..].split_inclusive(|byte| *byte == b'\n') {
                    if line.starts_with(prefix.as_bytes()) && cleanup.is_ok() {
                        return Ok(String::from_utf8_lossy(line).into_owned());
                    }
                }
                drop(bytes);
                let message = match cleanup {
                    Ok(()) => "fixture exited before readiness".to_owned(),
                    Err(error) => {
                        format!("fixture exited before readiness; cleanup also failed: {error}")
                    }
                };
                return Err(self.contextual(io::ErrorKind::UnexpectedEof, &message));
            }
            self.check_deadline()?;
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn leader_exited(&mut self) -> io::Result<bool> {
        #[cfg(target_os = "linux")]
        if let Some(descendants) = self.descendants.as_mut() {
            descendants.sample(false)?;
        }
        if self.status.is_some() {
            return Ok(true);
        }
        #[cfg(unix)]
        {
            use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
            let pid = i32::try_from(self.id())
                .ok()
                .and_then(Pid::from_raw)
                .ok_or_else(|| io::Error::other("invalid fixture process ID"))?;
            let result = loop {
                match waitid(
                    WaitId::Pid(pid),
                    WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
                ) {
                    Err(error) if error == rustix::io::Errno::INTR => continue,
                    result => break result.map_err(io::Error::from)?,
                }
            };
            Ok(result.is_some())
        }
        #[cfg(not(unix))]
        {
            self.status = self.child.inner_mut().try_wait()?;
            Ok(self.status.is_some())
        }
    }

    /// Observe SUT completion without fallback cleanup masking leaked descendants.
    pub fn observe_exit(&mut self) -> io::Result<()> {
        loop {
            if self.leader_exited()? {
                return Ok(());
            }
            self.check_deadline()?;
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn check_deadline(&self) -> io::Result<()> {
        if Instant::now() >= self.deadline {
            return Err(self.contextual(
                io::ErrorKind::TimedOut,
                "fixture exceeded its independent deadline",
            ));
        }
        Ok(())
    }

    fn kill_tree(&mut self) -> io::Result<()> {
        if self.terminated {
            return Ok(());
        }
        match self.child.start_kill() {
            Ok(()) => {}
            #[cfg(unix)]
            Err(error) if error.raw_os_error() == Some(nix::libc::ESRCH) => {}
            Err(error) => return Err(error),
        }
        self.terminated = true;
        Ok(())
    }

    pub fn terminate(&mut self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        let descendants = if self.terminated {
            Ok(())
        } else {
            self.descendants
                .as_mut()
                .map_or(Ok(()), descendants::Tracker::kill)
        };
        #[cfg(not(target_os = "linux"))]
        let descendants: io::Result<()> = Ok(());
        let killed = self.kill_tree();
        if killed.is_err() && self.child.inner_mut().kill().is_err() {
            return killed;
        }
        if self.status.is_none() {
            self.status = Some(self.child.inner_mut().wait()?);
        }
        let stdout = self.stdout.join();
        let stderr = self.stderr.join();
        descendants.and(killed).and(stdout).and(stderr)
    }

    pub fn wait(mut self) -> io::Result<Output> {
        self.observe_exit()?;
        self.terminate()?;
        let status = self
            .status
            .ok_or_else(|| io::Error::other("fixture leader status unavailable"))?;
        Ok(Output {
            status,
            stdout: self.stdout.take()?,
            stderr: self.stderr.take()?,
        })
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

pub fn capture(mut command: Command, timeout: Duration) -> io::Result<Output> {
    command.stdin(Stdio::null());
    Process::spawn(command, timeout)?.wait()
}

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(prefix: &str) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
}

impl std::ops::Deref for TempDir {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
