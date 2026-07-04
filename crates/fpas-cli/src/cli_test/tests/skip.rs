use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};
use std::time::Duration;

#[test]
fn test_cli_reports_skipped_tests_without_strict() {
    let cwd = create_temp_dir("fpas-test-skip");
    write_text(
        &cwd.join("skip_test.fpas"),
        "program S;\nuses Std.Test;\nbegin Skip('later') end.",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("SKIP  skip_test.fpas"));
    assert!(text.contains("1 skipped"));
}

#[test]
fn test_cli_reports_skipped_tests_with_timeout() {
    let cwd = create_temp_dir("fpas-test-skip-timeout");
    write_text(
        &cwd.join("skip_test.fpas"),
        "program S;\nuses Std.Test;\nbegin Skip('later') end.",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: Some(Duration::from_secs(1)),
            jobs: 1,
            strict: false,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("SKIP  skip_test.fpas"));
    assert!(text.contains("1 skipped"));
}

#[test]
fn test_cli_strict_fails_when_tests_are_skipped() {
    let cwd = create_temp_dir("fpas-test-strict-skip");
    write_text(
        &cwd.join("skip_test.fpas"),
        "program S;\nuses Std.Test;\nbegin Skip('later') end.",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: true,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 1);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("SKIP  skip_test.fpas"));
}
