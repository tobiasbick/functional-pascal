//! Normal worker scheduling of MCP handlers and their post-callback serving waits.
//!
//! Documentation: `docs/pascal/std/ai/mcp/README.md`.

#![allow(clippy::expect_used, reason = "process fixtures assert on CLI results")]

use std::{path::PathBuf, process::Command};

#[test]
fn serving_worker_resumes_after_callback_and_joins_when_server_closes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(root)
        .args(["run", "tests/fixtures/mcp_worker_server.fpas"])
        .output()
        .expect("MCP worker fixture must start");
    // The fixture's supervised child bounds deadlocks and prints no HTTP credentials.
    assert!(
        output.status.success(),
        "MCP worker lifecycle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "worker lifecycle verified"
    );
}
