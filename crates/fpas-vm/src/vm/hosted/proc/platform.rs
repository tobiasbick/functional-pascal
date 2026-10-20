//! Owned process trees and cancellable standard I/O, without local unsafe code.
//!
//! Unix descendants must remain in their inherited process group. Windows jobs
//! disallow breakaway. Passing pipe handles to unrelated processes is unsupported.

use std::io;
use std::process::{Command, ExitStatus};

use process_wrap::std::{ChildWrapper, CommandWrap};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub(super) use unix::{PipeReader, PipeWriter};
#[cfg(windows)]
pub(super) use windows::{PipeReader, PipeWriter};

pub(super) struct GroupChild {
    // Exactly one wrapper surrounds the native Child. Its inner wait operations
    // reap only the leader; the wrapper's own waits may also wait for descendants.
    child: Box<dyn ChildWrapper>,
    terminated: bool,
    status: Option<ExitStatus>,
    #[cfg(windows)]
    runtime: std::sync::Arc<tokio::runtime::Runtime>,
}

impl GroupChild {
    pub(super) fn spawn(command: Command) -> io::Result<Self> {
        #[cfg(unix)]
        unix::ensure_supported()?;
        #[cfg(windows)]
        let runtime = windows::runtime()?;

        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(process_wrap::std::ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(process_wrap::std::JobObject);

        Ok(Self {
            child: command.spawn()?,
            terminated: false,
            status: None,
            #[cfg(windows)]
            runtime,
        })
    }

    pub(super) fn take_stdin(&mut self) -> io::Result<Option<PipeWriter>> {
        let pipe = self.child.stdin().take();
        #[cfg(unix)]
        {
            pipe.map(unix::stdin).transpose()
        }
        #[cfg(windows)]
        {
            pipe.map(|pipe| windows::stdin(pipe, self.runtime.clone()))
                .transpose()
        }
    }

    pub(super) fn take_stdout(&mut self) -> io::Result<Option<PipeReader>> {
        let pipe = self.child.stdout().take();
        #[cfg(unix)]
        {
            pipe.map(unix::stdout).transpose()
        }
        #[cfg(windows)]
        {
            pipe.map(|pipe| windows::stdout(pipe, self.runtime.clone()))
                .transpose()
        }
    }

    pub(super) fn take_stderr(&mut self) -> io::Result<Option<PipeReader>> {
        let pipe = self.child.stderr().take();
        #[cfg(unix)]
        {
            pipe.map(unix::stderr).transpose()
        }
        #[cfg(windows)]
        {
            pipe.map(|pipe| windows::stderr(pipe, self.runtime.clone()))
                .transpose()
        }
    }

    /// Observe the leader, then close its owned tree so inherited pipes reach EOF.
    pub(super) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.status {
            self.terminate_group()?;
            return Ok(Some(status));
        }
        #[cfg(unix)]
        if !self.terminated {
            if !unix::leader_exited(self.child.id(), false)? {
                return Ok(None);
            }
            // WNOWAIT keeps the leader PID reserved until the group is signalled.
            // Reaping first could allow a later killpg to target a reused PGID.
            self.terminate_group()?;
        }
        self.status = self.child.inner_mut().try_wait()?;
        if self.status.is_some() {
            self.terminate_group()?;
        }
        Ok(self.status)
    }

    pub(super) fn terminate_group(&mut self) -> io::Result<()> {
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

    pub(super) fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.status {
            self.terminate_group()?;
            return Ok(status);
        }
        #[cfg(unix)]
        if !self.terminated {
            unix::leader_exited(self.child.id(), true)?;
            self.terminate_group()?;
        }
        let status = self.child.inner_mut().wait()?;
        self.status = Some(status);
        self.terminate_group()?;
        Ok(status)
    }
}

impl Drop for GroupChild {
    fn drop(&mut self) {
        // Keep the group/job and (on Windows) its pipe runtime alive until the
        // tree is terminated and the direct child has been reaped.
        if self.terminate_group().is_err() && self.child.inner_mut().start_kill().is_err() {
            return;
        }
        let _ = self.child.inner_mut().wait();
    }
}

#[cfg(all(test, unix))]
mod tests;
