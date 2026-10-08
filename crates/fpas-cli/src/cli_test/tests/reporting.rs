use crate::cli_input::{TestCliConfig, TestReportFormat};
use crate::cli_output::{DiagnosticFormat, Reporter};
use crate::cli_test::test_cli_with_stderr;
use crate::test_support::FailingWriter;
use crate::test_support::{create_temp_dir, write_text};

use super::super::report::{Summary, TestOutcome};
use super::super::runner::finish_test_run;

#[test]
fn test_cli_json_report_writes_summary_to_stdout() {
    let cwd = create_temp_dir("fpas-test-json");
    write_text(
        &cwd.join("ok_test.fpas"),
        "program O;\nuses Std.Test;\nbegin AssertTrue(true); end.",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: Some(TestReportFormat::Json),
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
    let json = String::from_utf8(stdout).expect("utf-8");
    assert!(json.contains("\"status\": \"pass\""));
    assert!(json.contains("ok_test.fpas"));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(!text.contains("Summary:"));
}

#[test]
fn json_program_stderr_is_retained_without_worker_isolation() {
    let cwd = create_temp_dir("fpas-test-program-stderr");
    let (command, arguments) = if cfg!(windows) {
        ("cmd", "['/C', 'echo child err>&2']")
    } else {
        ("sh", "['-c', 'echo child err >&2']")
    };
    for name in ["first_test.fpas", "second_test.fpas"] {
        write_text(
            &cwd.join(name),
            &format!(
                "program P; uses Std.Proc, Std.Console; begin case Run('{command}', {arguments}) of when Ok(const Code): WriteLn(Code); when Error(const Message): WriteLn(Message); end case; end."
            ),
        );
    }
    for jobs in [1, 2] {
        let mut config = report_config(TestReportFormat::Json);
        config.cwd = cwd.clone();
        config.input = crate::CliInput::SourceFile(cwd.clone());
        config.jobs = jobs;
        config.diagnostics = DiagnosticFormat::Json;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            test_cli_with_stderr(config, &mut stdout, &mut stderr),
            0,
            "{}",
            String::from_utf8_lossy(&stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&stdout).expect("report JSON");
        assert_eq!(report["summary"]["passed"], 2);
        let stderr = String::from_utf8(stderr).expect("stderr UTF-8");
        let records = stderr
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("stderr JSON"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2, "{records:?}");
        for record in records {
            assert_eq!(record["kind"], "program-output");
            assert_eq!(record["text"], "child err");
        }
    }
    std::fs::remove_dir_all(cwd).expect("remove fixtures");
}

#[test]
fn json_report_failure_is_reported_and_returns_nonzero() {
    let config = report_config(TestReportFormat::Json);
    let mut summary = Summary::default();
    summary.record("ok_test.fpas", TestOutcome::Pass);
    for mut stdout in [FailingWriter::immediately(), FailingWriter::after(8)] {
        let mut stderr = Vec::new();
        let exit_code = finish_test_run(
            &config,
            &summary,
            &mut stdout,
            &mut Reporter::new(DiagnosticFormat::Text, &mut stderr),
        );
        let stderr = String::from_utf8(stderr).expect("stderr must be UTF-8");

        assert_eq!(exit_code, 1);
        assert!(
            stderr.contains("Cannot write JSON test report to stdout"),
            "unexpected stderr: {stderr}"
        );
    }
}

#[test]
fn partial_summary_write_returns_nonzero() {
    let config = report_config_without_json();
    let mut summary = Summary::default();
    summary.record("ok_test.fpas", TestOutcome::Pass);
    for mut stderr in [FailingWriter::immediately(), FailingWriter::after(8)] {
        let mut stdout = Vec::new();
        let exit_code = finish_test_run(
            &config,
            &summary,
            &mut stdout,
            &mut Reporter::new(DiagnosticFormat::Text, &mut stderr),
        );

        assert_eq!(exit_code, 1);
    }
}

fn report_config(report: TestReportFormat) -> TestCliConfig {
    let mut config = report_config_without_json();
    config.report = Some(report);
    config
}

fn report_config_without_json() -> TestCliConfig {
    let cwd = std::path::PathBuf::from("unused-report-output-config");
    TestCliConfig {
        input: crate::CliInput::SourceFile(cwd.clone()),
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
    }
}
