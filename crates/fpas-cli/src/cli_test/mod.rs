//! `fpas test` — discover and run `*_test.fpas` programs.
//!
//! Spec: [`docs/pascal/10-projects.md`](../../../docs/pascal/10-projects.md),
//! [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md).

mod discover;
mod expect_stdout;
mod hooks;
mod parallel;
mod report;
mod run;
mod timeout;

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_input::{TestCliConfig, TestReportFormat};
use discover::{discover_test_files, filter_test_paths, is_test_file_name};
use fpas_project as project;
use report::{Summary, TestOutcome, print_json_report, print_summary};
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

    // List output is the command result and goes to stdout so it can be piped;
    // progress and summaries stay on stderr.
    if config.list_only {
        for path in &paths {
            let _ = writeln!(stdout, "{}", path.display());
        }
        return 0;
    }

    let _ = writeln!(stderr, "Running {} test(s)...", paths.len());
    let _ = writeln!(stderr);

    let job_count = parallel::effective_job_count(config.jobs, paths.len());
    if job_count <= 1 {
        return run_tests_sequential(config, paths, stdout, stderr);
    }

    run_tests_parallel(config, paths, stdout, stderr)
}

fn run_tests_sequential(
    config: TestCliConfig,
    paths: Vec<PathBuf>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let mut summary = Summary::default();
    for path in paths {
        let display = test_display_path(&path);
        let link = match link_context_for_test(&path) {
            Ok(Some(context)) => Some(context),
            Ok(None) => None,
            Err(message) => {
                let _ = writeln!(stderr, "  FAIL  {display}");
                let _ = writeln!(stderr, "        {message}");
                summary.record(&display, TestOutcome::CompileError);
                continue;
            }
        };
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

fn run_tests_parallel(
    config: TestCliConfig,
    paths: Vec<PathBuf>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let mut summary = Summary::default();
    let mut prepared = Vec::new();
    let mut preload_results = Vec::new();

    for (index, path) in paths.into_iter().enumerate() {
        let display = test_display_path(&path);
        match link_context_for_test(&path) {
            Ok(link) => prepared.push(parallel::PreparedTest {
                index,
                path,
                display,
                link,
            }),
            Err(message) => {
                let output = format!("  FAIL  {display}\n        {message}\n");
                preload_results.push(parallel::IndexedTestResult {
                    index,
                    display,
                    outcome: TestOutcome::CompileError,
                    output,
                });
            }
        }
    }

    let mut results = preload_results;
    results.extend(parallel::run_tests_parallel(
        prepared,
        config.jobs,
        config.script_path.as_deref(),
        config.timeout,
        config.fail_fast,
    ));
    results.sort_by_key(|result| result.index);

    for result in results {
        let _ = write!(stderr, "{}", result.output);
        if config.fail_fast && result.outcome.is_failure() {
            summary.record(&result.display, result.outcome);
            return finish_test_run(&config, &summary, stdout, stderr);
        }
        summary.record(&result.display, result.outcome);
    }

    finish_test_run(&config, &summary, stdout, stderr)
}

fn link_context_for_test(path: &Path) -> Result<Option<LinkContext>, String> {
    let Some(project_file) = find_enclosing_project(path) else {
        return Ok(None);
    };
    let loaded = project::load_project(&project_file)?;
    let hooks = hooks::discover_test_hooks(&loaded.source_files)?;
    Ok(Some(LinkContext {
        source_files: loaded.source_files,
        link_meta: loaded.link_meta,
        test_manifest: loaded.test_manifest,
        hooks,
    }))
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
    use std::time::Duration;

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
                jobs: 1,
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
                jobs: 1,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 0);
        let listed = String::from_utf8(stdout).expect("utf-8");
        assert!(listed.contains("one_test.fpas"));
        let progress = String::from_utf8(stderr).expect("utf-8");
        assert!(!progress.contains("FAIL"));
        assert!(!progress.contains("one_test.fpas"));
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
                jobs: 1,
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
                jobs: 1,
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
    fn test_cli_jobs_runs_tests_in_parallel_mode() {
        let cwd = create_temp_dir("fpas-test-jobs");
        write_text(
            &cwd.join("one_test.fpas"),
            "program O;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );
        write_text(
            &cwd.join("two_test.fpas"),
            "program T;\nuses Std.Test;\nbegin AssertEquals(2, 1 + 1) end.",
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
                jobs: 2,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("PASS  one_test.fpas"));
        assert!(text.contains("PASS  two_test.fpas"));
    }

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
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 0);
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
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 1);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("stdout mismatch"));
    }

    #[test]
    fn test_cli_rejects_unit_file_as_test_entry() {
        let cwd = create_temp_dir("fpas-test-unit-reject");
        write_text(
            &cwd.join("helper_test.fpas"),
            "unit Tests.Helper;\nprocedure X();\nbegin end;",
        );

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.join("helper_test.fpas")),
                cwd: cwd.clone(),
                fail_fast: false,
                list_only: false,
                script_path: None,
                filter: None,
                report: None,
                timeout: None,
                jobs: 1,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 2);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("must be `program` files"), "stderr={text}");
        assert!(text.contains("unit Tests.Helper"), "stderr={text}");
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
                jobs: 1,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 3);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("TIMEOUT  hang_test.fpas"));
    }
}
