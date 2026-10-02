//! Integration tests for `fpas test`.

use crate::cli_test::test_cli_with_stderr;
use crate::test_support::{create_temp_dir, write_text};
use crate::{CliInput, TestCliConfig};

#[test]
fn test_cli_runs_passing_tests_in_directory() {
    let cwd = create_temp_dir("fpas-test-pass");
    write_text(
        &cwd.join("math_test.fpas"),
        r#"program M;
uses Std.Test as Test;
begin Test.AssertEquals(6, 2 * 3); end program;"#,
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  math_test.fpas"));
}
