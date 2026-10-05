//! Captured standard output of test programs in the runner's result lines.

use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli_with_stderr;
use crate::test_support::{create_temp_dir, write_text};
use std::path::PathBuf;
use std::time::Duration;

fn config(
    cwd: PathBuf,
    show_output: bool,
    timeout: Option<Duration>,
    jobs: usize,
) -> TestCliConfig {
    TestCliConfig {
        input: crate::CliInput::SourceFile(cwd.clone()),
        cwd,
        fail_fast: false,
        list_only: false,
        script_path: None,
        filter: None,
        files: Vec::new(),
        report: None,
        timeout,
        jobs,
        strict: false,
        show_output,
        diagnostics: Default::default(),
        standard_library: None,
    }
}

fn run(name: &str, show_output: bool, timeout: Option<Duration>, jobs: usize) -> (i32, String) {
    let cwd = create_temp_dir(name);
    write_text(
        &cwd.join("pass_test.fpas"),
        "program P;\nuses Std.Console;\nbegin WriteLn('passing detail') end.",
    );
    write_text(
        &cwd.join("fail_test.fpas"),
        "program F;\nuses Std.Console, Std.Test;\nbegin WriteLn('failing detail'); AssertEquals(1, 2) end.",
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        config(cwd, show_output, timeout, jobs),
        &mut stdout,
        &mut stderr,
    );
    assert!(
        stdout.is_empty(),
        "test output must not reach the runner's stdout"
    );
    (exit, String::from_utf8(stderr).expect("utf-8"))
}

#[test]
fn failing_tests_always_show_their_standard_output() {
    let (exit, text) = run("fpas-test-output-default", false, None, 1);
    assert_eq!(exit, 1);
    assert!(
        text.contains("        stdout:\n          failing detail"),
        "{text}"
    );
    assert!(!text.contains("passing detail"), "{text}");
}

#[test]
fn show_output_prints_standard_output_of_passing_tests() {
    let (exit, text) = run("fpas-test-output-shown", true, None, 1);
    assert_eq!(exit, 1);
    assert!(
        text.contains("  PASS  pass_test.fpas\n        stdout:\n          passing detail"),
        "{text}"
    );
    assert!(text.contains("failing detail"), "{text}");
}

#[test]
fn show_output_reaches_isolated_parallel_workers() {
    let (exit, text) = run(
        "fpas-test-output-workers",
        true,
        Some(Duration::from_secs(60)),
        2,
    );
    assert_eq!(exit, 1);
    assert!(text.contains("passing detail"), "{text}");
    assert!(text.contains("failing detail"), "{text}");
}
