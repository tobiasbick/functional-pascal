//! Temporarily foreground an interactive Unix process group, without changing signal handlers.

use std::io;

use crate::execution::InteractiveProcessStdio;

pub(in crate::execution::vm::hosted::proc) struct Foreground {
    #[cfg(unix)]
    terminal: Option<std::fs::File>,
    #[cfg(unix)]
    previous: Option<nix::unistd::Pid>,
}

impl Foreground {
    pub(in crate::execution::vm::hosted::proc) fn new(
        streams: &mut InteractiveProcessStdio,
    ) -> io::Result<Self> {
        #[cfg(unix)]
        {
            use nix::unistd::{getpgrp, tcgetpgrp};
            let previous = streams.foreground.as_ref().map(tcgetpgrp).transpose()?;
            if previous.is_some_and(|group| group != getpgrp()) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Interactive host does not own the terminal foreground",
                ));
            }
            Ok(Self {
                terminal: streams.foreground.take(),
                previous,
            })
        }
        #[cfg(not(unix))]
        {
            let _ = streams;
            Ok(Self {})
        }
    }

    pub(in crate::execution::vm::hosted::proc) fn activate(&self, pid: u32) -> io::Result<()> {
        #[cfg(unix)]
        if let Some(terminal) = &self.terminal {
            use nix::sys::signal::{Signal, killpg};
            use nix::unistd::Pid;
            let raw = i32::try_from(pid).map_err(|_| io::Error::other("Invalid child PID"))?;
            let group = Pid::from_raw(raw);
            transfer(terminal, group)?;
            // The child may have attempted a terminal read before foreground transfer.
            match killpg(group, Signal::SIGCONT) {
                Ok(()) | Err(nix::errno::Errno::ESRCH) => {}
                Err(error) => return Err(error.into()),
            }
        }
        #[cfg(not(unix))]
        let _ = pid;
        Ok(())
    }

    pub(in crate::execution::vm::hosted::proc) fn restore(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        if let (Some(terminal), Some(previous)) = (&self.terminal, self.previous) {
            transfer(terminal, previous)?;
            self.previous = None;
        }
        Ok(())
    }
}

impl Drop for Foreground {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(unix)]
fn transfer(terminal: &std::fs::File, group: nix::unistd::Pid) -> io::Result<()> {
    use nix::sys::signal::{SigSet, SigmaskHow, Signal, pthread_sigmask};
    let mut blocked = SigSet::empty();
    blocked.add(Signal::SIGTTOU);
    let mut previous = SigSet::empty();
    pthread_sigmask(SigmaskHow::SIG_BLOCK, Some(&blocked), Some(&mut previous))?;
    let result = nix::unistd::tcsetpgrp(terminal, group);
    let restored = pthread_sigmask(SigmaskHow::SIG_SETMASK, Some(&previous), None);
    result.and(restored).map_err(io::Error::from)
}
