//! Sequential and parallel test-runner orchestration.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cli_input::{TestCliConfig, TestReportFormat};

use super::link::LinkContextCache;
use super::parallel;
use super::report::{Summary, TestOutcome, print_json_report, print_summary};
use super::run::{run_single_test, test_display_path};

pub(super) fn finish_test_run(
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

pub(super) fn run_tests_sequential(
    config: TestCliConfig,
    paths: Vec<PathBuf>,
    standard_library: Option<Arc<fpas_project::StandardLibrary>>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let mut summary = Summary::default();
    let mut links = LinkContextCache::new(standard_library);
    for (index, path) in paths.iter().enumerate() {
        let display = test_display_path(path).into_owned();
        let link = match links.context_for_test(path) {
            Ok(Some(context)) => Some(context),
            Ok(None) => None,
            Err(message) => {
                let _ = writeln!(stderr, "  FAIL  {display}");
                let _ = writeln!(stderr, "        {message}");
                summary.record(&display, TestOutcome::CompileError);
                if config.fail_fast {
                    record_not_run_tests(&mut summary, stderr, &paths[index + 1..]);
                    return finish_test_run(&config, &summary, stdout, stderr);
                }
                continue;
            }
        };
        let outcome = run_single_test(
            path,
            link.as_ref(),
            config.script_path.as_deref(),
            config.timeout,
            stderr,
        );
        summary.record(&display, outcome);
        if config.fail_fast && outcome.is_failure() {
            record_not_run_tests(&mut summary, stderr, &paths[index + 1..]);
            return finish_test_run(&config, &summary, stdout, stderr);
        }
    }

    finish_test_run(&config, &summary, stdout, stderr)
}

pub(super) fn run_tests_parallel(
    config: TestCliConfig,
    paths: Vec<PathBuf>,
    standard_library: Option<Arc<fpas_project::StandardLibrary>>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let mut summary = Summary::default();
    let mut prepared = Vec::new();
    let mut preload_results = Vec::new();
    let mut links = LinkContextCache::new(standard_library);

    for (index, path) in paths.into_iter().enumerate() {
        let display = test_display_path(&path).into_owned();
        match links.context_for_test(&path) {
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
        summary.record(&result.display, result.outcome);
    }

    finish_test_run(&config, &summary, stdout, stderr)
}

fn record_not_run_tests(summary: &mut Summary, stderr: &mut dyn Write, paths: &[PathBuf]) {
    for path in paths {
        let display = test_display_path(path).into_owned();
        let _ = writeln!(stderr, "  ---  {display} (not run, --fail-fast)");
        summary.record(&display, TestOutcome::NotRun);
    }
}
