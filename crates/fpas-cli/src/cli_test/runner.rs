//! Sequential and parallel test-runner orchestration.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cli_input::{TestCliConfig, TestReportFormat};

use super::image::attach_test_images;
use super::link::LinkContextCache;
use super::parallel;
use super::report::{Summary, TestOutcome, print_json_report, print_summary};
use super::run::{run_single_test_prepared, test_display_path};

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
    let mut prepared = Vec::with_capacity(paths.len());
    let mut link_errors = HashMap::<usize, String>::new();
    for (index, path) in paths.iter().enumerate() {
        let display = test_display_path(path).into_owned();
        match links.context_for_test(path) {
            Ok(link) => prepared.push(parallel::PreparedTest {
                index,
                path: path.clone(),
                display,
                link,
                compiled: None,
            }),
            Err(message) => {
                link_errors.insert(index, message);
            }
        }
    }
    attach_test_images(&mut prepared);
    let mut prepared = prepared
        .into_iter()
        .map(|test| (test.index, test))
        .collect::<HashMap<_, _>>();

    for (index, path) in paths.iter().enumerate() {
        let display = test_display_path(path).into_owned();
        let Some(test) = prepared.remove(&index) else {
            if let Some(message) = link_errors.remove(&index) {
                let _ = writeln!(stderr, "  FAIL  {display}");
                let _ = writeln!(stderr, "        {message}");
                summary.record(&display, TestOutcome::CompileError);
                if config.fail_fast {
                    record_not_run_tests(&mut summary, stderr, &paths[index + 1..]);
                    return finish_test_run(&config, &summary, stdout, stderr);
                }
                continue;
            }
            continue;
        };
        let outcome = run_single_test_prepared(
            &test.path,
            test.link.as_ref(),
            config.script_path.as_deref(),
            config.timeout,
            stderr,
            test.compiled.as_ref(),
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

    for (index, path) in paths.iter().enumerate() {
        let display = test_display_path(path).into_owned();
        match links.context_for_test(path) {
            Ok(link) => prepared.push(parallel::PreparedTest {
                index,
                path: path.clone(),
                display,
                link,
                compiled: None,
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

    if config.fail_fast
        && let Some(first_error) = preload_results.iter().map(|result| result.index).min()
    {
        prepared.retain(|test| test.index < first_error);
        preload_results.retain(|result| result.index == first_error);
        preload_results.extend(paths.iter().enumerate().skip(first_error + 1).map(
            |(index, path)| {
                parallel::not_run_result_for(index, test_display_path(path).into_owned())
            },
        ));
    }

    attach_test_images(&mut prepared);

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
