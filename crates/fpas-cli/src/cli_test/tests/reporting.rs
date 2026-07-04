use crate::cli_input::{TestCliConfig, TestReportFormat};
use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};

#[test]
fn test_cli_json_report_writes_summary_to_stdout() {
    let cwd = create_temp_dir("fpas-test-json");
    write_text(
        &cwd.join("ok_test.fpas"),
        "program O;\nuses Std.Test;\nbegin AssertTrue(true) end.",
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
            report: Some(TestReportFormat::Json),
            timeout: None,
            jobs: 1,
            strict: false,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let json = String::from_utf8(stdout).expect("utf-8");
    assert!(json.contains("\"status\": \"pass\""));
    assert!(json.contains("ok_test.fpas"));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(!text.contains("Summary:"));
}
