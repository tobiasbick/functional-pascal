//! Sequential and parallel test-runner orchestration.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cli_input::{TestCliConfig, TestReportFormat};
use crate::cli_output::{CliFailure, Reporter};

use super::image::attach_test_images;
use super::link::LinkContextCache;
use super::log;
use super::parallel;
use super::report::{Summary, TestOutcome, print_json_report, print_summary};
use super::run::{TestRunSettings, run_single_test_prepared, test_display_path};

pub(super) fn finish_test_run(
    config: &TestCliConfig,
    summary: &Summary,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    if config.report == Some(TestReportFormat::Json) {
        if let Err(error) = print_json_report(stdout, summary) {
            return reporter.failure(&crate::cli_output::write_failure(
                "JSON test report to stdout",
                &error,
            ));
        }
    } else if let Some(stream) = reporter.text_stream()
        && let Err(error) = print_summary(stream, summary)
    {
        return reporter.failure(&crate::cli_output::write_failure(
            "test summary to stderr",
            &error,
        ));
    }
    summary.exit_code(config.strict)
}

pub(super) fn run_tests_sequential(
    config: TestCliConfig,
    paths: Vec<PathBuf>,
    standard_library: Option<Arc<fpas_project::StandardLibrary>>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let mut summary = Summary::default();
    let mut links = LinkContextCache::new(standard_library);
    let mut prepared = Vec::with_capacity(paths.len());
    let mut link_errors = HashMap::<usize, CliFailure>::new();
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
            Err(failure) => {
                link_errors.insert(index, failure);
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
            if let Some(failure) = link_errors.remove(&index) {
                log::banner(reporter, "FAIL", &display);
                log::failure(reporter, &failure);
                summary.record(&path.to_string_lossy(), TestOutcome::CompileError);
                if config.fail_fast {
                    record_not_run_tests(&mut summary, reporter, &paths[index + 1..]);
                    return finish_test_run(&config, &summary, stdout, reporter);
                }
            }
            continue;
        };
        let outcome = run_single_test_prepared(
            &test.path,
            test.link.as_ref(),
            TestRunSettings {
                script_override: config.script_path.as_deref(),
                timeout: config.timeout,
                show_output: config.show_output,
                diagnostics: config.diagnostics,
            },
            reporter,
            test.compiled.as_ref(),
        );
        summary.record(&path.to_string_lossy(), outcome);
        if config.fail_fast && outcome.is_failure() {
            record_not_run_tests(&mut summary, reporter, &paths[index + 1..]);
            return finish_test_run(&config, &summary, stdout, reporter);
        }
    }

    finish_test_run(&config, &summary, stdout, reporter)
}

pub(super) fn run_tests_parallel(
    config: TestCliConfig,
    paths: Vec<PathBuf>,
    standard_library: Option<Arc<fpas_project::StandardLibrary>>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
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
            Err(failure) => {
                let output = parallel::render_output(config.diagnostics, |reporter| {
                    log::banner(reporter, "FAIL", &display);
                    log::failure(reporter, &failure);
                });
                preload_results.push(parallel::IndexedTestResult {
                    index,
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
                parallel::not_run_result_for(
                    index,
                    test_display_path(path).into_owned(),
                    config.diagnostics,
                )
            },
        ));
    }

    attach_test_images(&mut prepared);

    let mut results = preload_results;
    results.extend(parallel::run_tests_parallel(
        prepared,
        config.jobs,
        TestRunSettings {
            script_override: config.script_path.as_deref(),
            timeout: config.timeout,
            show_output: config.show_output,
            diagnostics: config.diagnostics,
        },
        config.fail_fast,
    ));
    results.sort_by_key(|result| result.index);

    for result in results {
        reporter.forward(result.output.as_bytes());
        summary.record(&paths[result.index].to_string_lossy(), result.outcome);
    }

    finish_test_run(&config, &summary, stdout, reporter)
}

fn record_not_run_tests(summary: &mut Summary, reporter: &mut Reporter<'_>, paths: &[PathBuf]) {
    for path in paths {
        let display = test_display_path(path).into_owned();
        log::banner(
            reporter,
            "---",
            &format!("{display} (not run, --fail-fast)"),
        );
        summary.record(&path.to_string_lossy(), TestOutcome::NotRun);
    }
}
