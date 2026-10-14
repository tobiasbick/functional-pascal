//! End-to-end coverage for generated project query workloads.

use std::error::Error;
use std::path::Path;
use std::process::Command;

#[test]
fn project_queries_analyzes_generated_imported_units() -> Result<(), Box<dyn Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-bench"))
        .args(["native", "language-service-project", "1", "2", "warm"])
        .current_dir(workspace)
        .output()?;

    assert!(
        output.status.success(),
        "project query workload failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
