//! Runs the FPAS IDE test project under `apps/ide/tests/`.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("fpas-cli crate must live two levels below the repository root")
        .to_path_buf()
}

#[test]
fn ide_tests_pass() {
    let root = repo_root();
    let (exit, _, stderr) = super::support::run_cli_args_and_capture_output(
        &[
            String::from("test"),
            String::from("apps/ide/tests/ide-tests.fpasprj"),
        ],
        &root,
    );

    assert_eq!(
        exit, 0,
        "fpas test apps/ide/tests/ide-tests.fpasprj failed\nstderr:\n{stderr}"
    );
}
