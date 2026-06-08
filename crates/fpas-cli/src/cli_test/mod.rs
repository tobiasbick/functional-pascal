//! `fpas test` — discover and run `*_test.fpas` programs.
//!
//! Spec: [`docs/pascal/10-projects.md`](../../../docs/pascal/10-projects.md),
//! [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md).

mod discover;
mod report;
mod run;
mod timeout;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cli_input::{TestCliConfig, TestReportFormat};
use discover::{discover_test_files, filter_test_paths, is_test_file_name};
use fpas_project as project;
use report::{Summary, print_json_report, print_summary};
use run::{LinkContext, run_single_test};

fn test_display_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(String::from)
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn finish_test_run(
    config: &TestCliConfig,
    summary: &Summary,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    if config.report == Some(TestReportFormat::Json) {
        let _ = print_json_report(stdout, summary);
    } else {
        let _ = print_summary(stderr, summary);
    }
    summary.exit_code()
}

/// Runs discovered tests and prints a pass/fail summary.
pub(crate) fn test_cli(
    config: TestCliConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let mut paths = match discover_test_files(&config.input, config.cwd.as_path()) {
        Ok(paths) => paths,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 2;
        }
    };

    if let Some(filter) = config.filter.as_deref() {
        paths = filter_test_paths(paths, filter);
        if paths.is_empty() {
            let _ = writeln!(
                stderr,
                "No test files matched filter `{filter}`.\n  help: `--filter` is a case-insensitive substring on the test file path."
            );
            return 2;
        }
    } else if paths.is_empty() {
        let _ = writeln!(
            stderr,
            "No test files found (expected `*_test.fpas`).\n  help: Pass a directory, project, or single test file."
        );
        return 2;
    }

    if config.list_only {
        for path in &paths {
            let _ = writeln!(stderr, "{}", path.display());
        }
        return 0;
    }

    let _ = writeln!(stderr, "Running {} test(s)...", paths.len());
    let _ = writeln!(stderr);

    let mut summary = Summary::default();
    for path in paths {
        let display = test_display_path(&path);
        let link = link_context_for_test(&path);
        let outcome = run_single_test(
            &path,
            link.as_ref(),
            config.script_path.as_deref(),
            config.timeout,
            stderr,
        );
        if config.fail_fast && outcome.is_failure() {
            summary.record(&display, outcome);
            return finish_test_run(&config, &summary, stdout, stderr);
        }
        summary.record(&display, outcome);
    }

    finish_test_run(&config, &summary, stdout, stderr)
}

fn link_context_for_test(path: &Path) -> Option<LinkContext> {
    let project_file = find_enclosing_project(path)?;
    let loaded = project::load_project(&project_file).ok()?;
    Some(LinkContext {
        source_files: loaded.source_files,
        link_meta: loaded.link_meta,
        test_manifest: loaded.test_manifest,
    })
}

fn find_enclosing_project(start: &Path) -> Option<PathBuf> {
    let mut dir = start.parent()?.to_path_buf();
    loop {
        if let Ok(read_dir) = std::fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("fpasprj"))
                {
                    return Some(path);
                }
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Validates that an explicit single-file test target looks like a test program.
pub(crate) fn validate_explicit_test_file(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        return Ok(());
    }
    if !is_test_file_name(path) {
        return Err(format!(
            "`{}` is not a test file.\n  help: Test files must be named `*_test.fpas`.",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn test_cli_runs_matching_tests_in_directory() {
        let cwd = create_temp_dir("fpas-test-dir");
        write_text(
            &cwd.join("pass_test.fpas"),
            "program P;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );
        write_text(
            &cwd.join("fail_test.fpas"),
            "program F;\nuses Std.Test;\nbegin AssertTrue(false) end.",
        );
        write_text(&cwd.join("helper.fpas"), "unit H;\nprocedure X; begin end;");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.clone()),
                cwd: cwd.clone(),
                fail_fast: false,
                list_only: false,
                script_path: None,
                filter: None,
                report: None,
                timeout: None,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 1);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("PASS  pass_test.fpas"));
        assert!(text.contains("FAIL  fail_test.fpas"));
        assert!(!text.contains("helper.fpas"));
    }

    #[test]
    fn test_cli_list_only_prints_paths_without_running() {
        let cwd = create_temp_dir("fpas-test-list");
        write_text(
            &cwd.join("one_test.fpas"),
            "program O;\nuses Std.Test;\nbegin AssertTrue(false) end.",
        );

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.clone()),
                cwd,
                fail_fast: false,
                list_only: true,
                script_path: None,
                filter: None,
                report: None,
                timeout: None,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 0);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("one_test.fpas"));
        assert!(!text.contains("FAIL"));
    }

    #[test]
    fn test_cli_filter_runs_matching_tests_only() {
        let cwd = create_temp_dir("fpas-test-filter");
        write_text(
            &cwd.join("menu_test.fpas"),
            "program M;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );
        write_text(
            &cwd.join("other_test.fpas"),
            "program O;\nuses Std.Test;\nbegin AssertTrue(false) end.",
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
                filter: Some("menu".to_string()),
                report: None,
                timeout: None,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 0);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("PASS  menu_test.fpas"));
        assert!(!text.contains("other_test.fpas"));
    }

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
    fn test_cli_timeout_aborts_infinite_loop() {
        let cwd = create_temp_dir("fpas-test-timeout");
        write_text(
            &cwd.join("hang_test.fpas"),
            "program H;\nbegin\n  while 1 = 1 do\n  begin\n  end\nend.",
        );

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.join("hang_test.fpas")),
                cwd: cwd.clone(),
                fail_fast: false,
                list_only: false,
                script_path: None,
                filter: None,
                report: None,
                timeout: Some(Duration::from_secs(1)),
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 3);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("TIMEOUT  hang_test.fpas"));
    }
}
