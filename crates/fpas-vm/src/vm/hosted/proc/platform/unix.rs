use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::process::{ChildStderr, ChildStdin, ChildStdout};

use nix::fcntl::{FcntlArg, OFlag, fcntl};

pub(in crate::vm::hosted::proc) struct PipeWriter(ChildStdin);

pub(in crate::vm::hosted::proc) enum PipeReader {
    Stdout(ChildStdout),
    Stderr(ChildStderr),
}

pub(super) fn stdin(pipe: ChildStdin) -> io::Result<PipeWriter> {
    nonblocking(&pipe)?;
    Ok(PipeWriter(pipe))
}

pub(super) fn stdout(pipe: ChildStdout) -> io::Result<PipeReader> {
    nonblocking(&pipe)?;
    Ok(PipeReader::Stdout(pipe))
}

pub(super) fn stderr(pipe: ChildStderr) -> io::Result<PipeReader> {
    nonblocking(&pipe)?;
    Ok(PipeReader::Stderr(pipe))
}

fn nonblocking(pipe: &impl AsFd) -> io::Result<()> {
    let flags = fcntl(pipe, FcntlArg::F_GETFL).map_err(io::Error::from)?;
    let flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
    fcntl(pipe, FcntlArg::F_SETFL(flags)).map_err(io::Error::from)?;
    Ok(())
}

impl Read for PipeReader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(pipe) => pipe.read(bytes),
            Self::Stderr(pipe) => pipe.read(bytes),
        }
    }
}

impl Write for PipeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub(super) fn ensure_supported() -> io::Result<()> {
    #[cfg(any(
        target_os = "cygwin",
        target_os = "horizon",
        target_os = "openbsd",
        target_os = "redox"
    ))]
    return Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Owned processes require waitid with WNOWAIT",
    ));
    #[cfg(not(any(
        target_os = "cygwin",
        target_os = "horizon",
        target_os = "openbsd",
        target_os = "redox"
    )))]
    Ok(())
}

pub(super) fn leader_exited(pid: u32, blocking: bool) -> io::Result<bool> {
    #[cfg(any(
        target_os = "cygwin",
        target_os = "horizon",
        target_os = "openbsd",
        target_os = "redox"
    ))]
    {
        let _ = (pid, blocking);
        ensure_supported()?;
        unreachable!()
    }
    #[cfg(not(any(
        target_os = "cygwin",
        target_os = "horizon",
        target_os = "openbsd",
        target_os = "redox"
    )))]
    {
        use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};

        let pid = i32::try_from(pid)
            .ok()
            .and_then(Pid::from_raw)
            .ok_or_else(|| io::Error::other("Invalid owned process ID"))?;
        let mut options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT;
        if !blocking {
            options |= WaitIdOptions::NOHANG;
        }
        loop {
            match waitid(WaitId::Pid(pid), options) {
                Err(error) if error == rustix::io::Errno::INTR => continue,
                result => {
                    return result
                        .map(|status| status.is_some())
                        .map_err(io::Error::from);
                }
            }
        }
    }
}
