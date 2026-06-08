//! Integration tests for `fpas test` with test projects.

use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};
use crate::{CliInput, TestCliConfig};

#[test]
fn test_cli_runs_tests_from_test_project_file() {
    let cwd = create_temp_dir("fpas-test-project");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("alpha_test.fpas"),
        "program A;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );
    write_text(
        &cwd.join("beta_test.fpas"),
        "program B;\nuses Std.Test;\nbegin AssertEquals(2, 1 + 1) end.",
    );

    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
        },
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  alpha_test.fpas"));
    assert!(text.contains("PASS  beta_test.fpas"));
}
