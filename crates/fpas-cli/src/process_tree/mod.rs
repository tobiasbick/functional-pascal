//! Owned worker process containment shared by tests and batch debugging.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::io;
use std::process::{Child, Command, ExitStatus};

/// Places a worker and its descendants in a separately terminable process tree.
pub(crate) fn configure(command: &mut Command) -> io::Result<()> {
    #[cfg(unix)]
    {
        unix::configure(command)
    }
    #[cfg(windows)]
    {
        windows::configure(command);
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = command;
        Ok(())
    }
}

/// Keep the Unix session leader waitable until its entire session is cleaned up.
pub(crate) fn try_wait_owned(child: &mut Child) -> io::Result<Option<ExitStatus>> {
    #[cfg(unix)]
    {
        unix::try_wait_owned(child)
    }
    #[cfg(not(unix))]
    {
        child.try_wait()
    }
}

/// Terminates the owned tree and reaps the direct child.
pub(crate) fn terminate(child: &mut Child) -> io::Result<()> {
    #[cfg(unix)]
    {
        unix::terminate(child)
    }
    #[cfg(windows)]
    {
        windows::terminate(child);
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = child.kill();
        child.wait().map(|_| ())
    }
}

/// Locates the installed CLI, including an ordinary Cargo test executable's sibling.
pub(crate) fn executable() -> std::io::Result<std::path::PathBuf> {
    let current = std::env::current_exe()?;
    if current
        .parent()
        .and_then(std::path::Path::file_name)
        .is_some_and(|name| name == "deps")
        && let Some(profile) = current.parent().and_then(std::path::Path::parent)
    {
        let candidate = profile.join(if cfg!(windows) { "fpas.exe" } else { "fpas" });
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Ok(current)
}
