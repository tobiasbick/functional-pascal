//! Windows process-tree termination for isolated test workers.

use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};

const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub(super) fn configure(command: &mut Command) {
    command.creation_flags(CREATE_NEW_PROCESS_GROUP);
}

pub(super) fn terminate(child: &mut Child) {
    let pid = child.id().to_string();
    let _ = Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
    let _ = child.wait();
}
