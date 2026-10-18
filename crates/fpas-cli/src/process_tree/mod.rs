//! Owned worker process containment shared by tests and batch debugging.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::process::{Child, Command};

/// Places a worker and its descendants in a separately terminable process tree.
pub(crate) fn configure(command: &mut Command) {
    #[cfg(unix)]
    unix::configure(command);
    #[cfg(windows)]
    windows::configure(command);
}

/// Terminates the owned tree and reaps the direct child.
pub(crate) fn terminate(child: &mut Child) {
    #[cfg(unix)]
    unix::terminate(child);
    #[cfg(windows)]
    windows::terminate(child);
    #[cfg(not(any(unix, windows)))]
    {
        let _ = child.kill();
        let _ = child.wait();
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
