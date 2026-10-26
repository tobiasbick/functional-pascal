//! Blocking sockets must not stop the all-stop task scheduler or its cancellation path.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

#![allow(clippy::expect_used, reason = "process fixtures assert on CLI results")]

use std::{path::PathBuf, process::Command};

#[test]
fn debugger_runs_socket_peers_and_joins_a_cancelled_accept() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(root)
        .args([
            "debug",
            "tests/fixtures/debug_network_waits.fpas",
            "--timeout",
            "5",
        ])
        .output()
        .expect("debug network fixture must start");
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("batch debugger must publish a JSON result");
    assert!(output.status.success(), "{report}");
    assert_eq!(report["outcome"], "completed");
    let rendered = report["output"].to_string();
    assert!(rendered.contains("nested accept joined"));
    assert!(rendered.contains("network exchange completed"));
}
