//! `fpas test` — discover and run `*_test.fpas` programs.
//!
//! Spec: [`docs/pascal/10-projects.md`](../../../docs/pascal/10-projects.md),
//! [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md).

mod discover;
mod expect_pixels;
mod expect_stdout;
mod hooks;
mod parallel;
mod report;
mod run;
mod timeout;

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_input::{TestCliConfig, TestReportFormat};
use discover::{discover_test_files, filter_test_paths};
use fpas_project as project;
use report::{Summary, TestOutcome, print_json_report, print_summary};
use run::{LinkContext, run_single_test, test_display_path};

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
    summary.exit_code(config.strict)
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
        let display = test_display_path(&path).into_owned();
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
        let display = test_display_path(&path).into_owned();
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
    let Some(project_file) = find_enclosing_project(path)? else {
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

fn find_enclosing_project(start: &Path) -> Result<Option<PathBuf>, String> {
    use crate::cli_paths::{PROJECT_FILE_EXTENSION, has_extension};

    let mut dir = start
        .parent()
        .ok_or_else(|| {
            format!(
                "Cannot resolve enclosing project for `{}`.",
                start.display()
            )
        })?
        .to_path_buf();
    loop {
        let mut candidates = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_file() && has_extension(&path, PROJECT_FILE_EXTENSION) {
                    candidates.push(path);
                }
            }
        }
        candidates.sort();
        match candidates.len() {
            0 => {}
            1 => return Ok(Some(candidates.remove(0))),
            _ => {
                let entries = candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "Found multiple `.fpasprj` files in `{}`: {entries}.\n  help: Keep one project manifest per directory or pass an explicit `.fpasprj` path.",
                    dir.display()
                ));
            }
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

/// Validates that an explicit single-file test target looks like a test program.
pub(crate) fn validate_explicit_test_file(path: &Path) -> Result<(), String> {
    if !project::is_test_source_file(path) {
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
                strict: false,
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
                strict: false,
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
                strict: false,
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
                strict: false,
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
    fn test_cli_compares_golden_pixels_for_headless_graph() {
        let cwd = create_temp_dir("fpas-test-expect-pixels");
        write_text(
            &cwd.join("graph_test.fpas"),
            "program G;\nuses Std.Console, Std.Graph, Std.Test;\n\
             mutable var QuitSeen: boolean := false;\n\
             procedure OnPaint(App: Application);\n\
             begin\n\
               Application.Clear(App, $00020408);\n\
               Application.DrawText(App, 2, 2, 'FPAS', $00FFFFFF);\n\
               Application.Present(App)\n\
             end;\n\
             function OnKeyPressed(App: Application; Key: KeyEvent): boolean;\n\
             begin\n\
               if Key.kind = KeyKind.Escape then\n\
               begin\n\
                 QuitSeen := true;\n\
                 Application.HostRequestQuit(App);\n\
                 return true\n\
               end;\n\
               return false\n\
             end;\n\
             begin\n\
               var App: Application := Application.OpenForTest(32, 24);\n\
               var EscapeKey: Std.Console.KeyEvent := record\n\
                 kind := Std.Console.KeyKind.Escape;\n\
                 ch := #27;\n\
                 shift := false;\n\
                 ctrl := false;\n\
                 alt := false;\n\
                 meta := false;\n\
               end;\n\
               var Handlers: ApplicationHandlers := record\n\
                 OnPaint := OnPaint;\n\
                 OnKeyPressed := Some(OnKeyPressed);\n\
               end;\n\
               Application.Configure(App, Handlers);\n\
               Application.TestSendKey(App, EscapeKey);\n\
               Application.Run(App);\n\
               AssertTrue(QuitSeen)\n\
             end.",
        );
        write_text(
            &cwd.join("graph_test.expect.pixels"),
            "# size 32 24\n0 0 0x00020408\n2 2 0x00FFFFFF\n",
        );

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.join("graph_test.fpas")),
                cwd: cwd.clone(),
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

        assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("PASS  graph_test.fpas"));
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
                strict: false,
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
                strict: false,
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
                strict: false,
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
                strict: false,
            },
            &mut stdout,
            &mut stderr,
        );

        assert_eq!(exit, 3);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("TIMEOUT  hang_test.fpas"));
    }

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
}
