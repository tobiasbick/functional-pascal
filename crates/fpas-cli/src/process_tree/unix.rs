//! Unix process-group containment for isolated test workers.

use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

pub(super) fn configure(command: &mut Command) {
    command.process_group(0);
}

pub(super) fn terminate(child: &mut Child) {
    let group = format!("-{}", child.id());
    let _ = Command::new("kill")
        .args(["-KILL", "--", &group])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
    let _ = child.wait();
}
