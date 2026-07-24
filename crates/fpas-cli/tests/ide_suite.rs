//! Runs IDE FPAS tests through the real CLI executable.
//!
//! The IDE process integration invokes `CurrentExecutable`, so an in-process
//! `run_cli` test would incorrectly expose the Rust test harness as the compiler.

use std::path::Path;
use std::process::Command;

#[test]
fn fpas_suite_ide() -> std::io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .args(["test", "--std-lib", "lib", "tests/ide/ide-tests.fpasprj"])
        .current_dir(root)
        .output()?;
    assert!(
        output.status.success(),
        "fpas IDE suite failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
