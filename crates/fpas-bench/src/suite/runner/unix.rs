//! Unix process-group containment for benchmark processes.

use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

pub(super) fn configure(command: &mut Command) {
    command.process_group(0);
}

pub(super) fn terminate(child: &mut Child) -> (Option<String>, Option<io::Error>) {
    let group = format!("-{}", child.id());
    let termination_error = Command::new("kill")
        .args(["-KILL", "--", &group])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| error.to_string())
        .and_then(|status| {
            status
                .success()
                .then_some(())
                .ok_or_else(|| format!("`kill` exited with {status}"))
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
