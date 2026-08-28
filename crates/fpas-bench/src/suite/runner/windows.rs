//! Windows process-tree containment for benchmark processes.

use std::io;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};

const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub(super) fn configure(command: &mut Command) {
    command.creation_flags(CREATE_NEW_PROCESS_GROUP);
}

pub(super) fn terminate(child: &mut Child) -> (Option<String>, Option<io::Error>) {
    let pid = child.id().to_string();
    let termination_error = Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| error.to_string())
        .and_then(|status| {
            status
                .success()
                .then_some(())
                .ok_or_else(|| format!("`taskkill` exited with {status}"))
        })
        .err();
    let fallback_error = termination_error
        .as_ref()
        .and_then(|_| child.kill().err())
        .map(|error| error.to_string());
    let termination_error = match (termination_error, fallback_error) {
        (Some(tree), Some(fallback)) => Some(format!("{tree}; direct kill failed: {fallback}")),
        (tree, _) => tree,
    };
    let wait_error = child.wait().err();
    (termination_error, wait_error)
}
