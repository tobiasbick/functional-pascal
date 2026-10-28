//! Diagnostics from a runner invoked without an appended program image.

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use std::process::Command;

#[test]
fn runner_startup_failure_is_a_positionless_json_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-runner"))
        .env("FPAS_DIAGNOSTICS", "json")
        .output()
        .expect("runner starts");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let record: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("one JSON diagnostic");
    assert_eq!(record["code"], "FP4125");
    assert_eq!(record["location"], serde_json::Value::Null);
}
