use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};

#[test]
fn test_cli_compares_golden_stdout_sidecar() {
    let cwd = create_temp_dir("fpas-test-expect-stdout");
    write_text(
        &cwd.join("echo_test.fpas"),
        "program E;\nuses Std.Console, Std.Test;\nbegin WriteLn('Hello'); WriteLn('World') end.",
    );
    write_text(&cwd.join("echo_test.expect.stdout"), "Hello\nWorld\n");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("echo_test.fpas")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  echo_test.fpas"));
}

#[test]
fn test_cli_fails_on_stdout_mismatch() {
    let cwd = create_temp_dir("fpas-test-expect-stdout-fail");
    write_text(
        &cwd.join("echo_test.fpas"),
        "program E;\nuses Std.Console, Std.Test;\nbegin WriteLn('Hi') end.",
    );
    write_text(&cwd.join("echo_test.expect.stdout"), "Hello\n");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("echo_test.fpas")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 1);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("stdout mismatch"));
}
