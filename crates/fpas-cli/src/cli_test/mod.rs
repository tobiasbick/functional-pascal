//! `fpas test` — discover and run `*_test.fpas` programs.
//!
//! Spec: [`docs/pascal/program-structure/cli.md`](../../../docs/pascal/program-structure/cli.md),
//! [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md).

mod discover;
mod expect_stdout;
mod hooks;
mod image;
mod link;
mod log;
mod parallel;
mod process;
mod report;
mod run;
mod runner;
mod scratch;

#[cfg(test)]
mod tests;

use std::io::Write;

use crate::cli_input::TestCliConfig;
use crate::cli_output::{CliFailure, Reporter};
use discover::{discover_test_files, filter_test_paths};
use fpas_diagnostics::codes::{CLI_ARGUMENTS_INVALID, CLI_INPUT_UNSUPPORTED};
use fpas_project as project;
use runner::{run_tests_parallel, run_tests_sequential};
use std::path::Path;
use std::sync::Arc;

use parallel::effective_job_count;

/// Runs the private test-process protocol before normal CLI parsing.
pub(crate) fn run_process_worker(args: &[String]) -> Option<i32> {
    process::run_worker_from_args(args)
}

/// Runs discovered tests and prints a pass/fail summary.
pub(crate) fn test_cli(
    config: TestCliConfig,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let standard_library =
        match crate::standard_library::resolve_standard_library(config.standard_library.as_deref())
        {
            Ok(library) => library.map(Arc::new),
            Err(failure) => return reporter.failure(&failure),
        };
    let mut paths = match discover_test_files(&config.input, config.cwd.as_path()) {
        Ok(paths) => paths,
        Err(failure) => {
            reporter.failure(&failure);
            return 2;
        }
    };

    if let Some(filter) = config.filter.as_deref() {
        paths = filter_test_paths(paths, filter);
        if paths.is_empty() {
            reporter.failure(
                &CliFailure::new(
                    CLI_ARGUMENTS_INVALID,
                    format!("No test files matched filter `{filter}`."),
                )
                .with_help("`--filter` is a case-insensitive substring on the test file path."),
            );
            return 2;
        }
    } else if paths.is_empty() {
        reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                "No test files found (expected `*_test.fpas`).",
            )
            .with_help("Pass a directory, project, or single test file."),
        );
        return 2;
    }

    if !config.files.is_empty() {
        paths = match discover::select_test_paths(paths, &config.files, &config.cwd) {
            Ok(paths) => paths,
            Err(failure) => {
                reporter.failure(&failure);
                return 2;
            }
        };
    }

    // List output is the command result and goes to stdout so it can be piped;
    // progress and summaries stay on stderr.
    if config.list_only {
        for path in &paths {
            if let Err(error) = writeln!(stdout, "{}", path.display()) {
                return reporter.failure(&crate::cli_output::write_failure(
                    "test file list to stdout",
                    &error,
                ));
            }
        }
        return 0;
    }

    reporter.progress(format_args!("Running {} test(s)...\n", paths.len()));

    let job_count = effective_job_count(config.jobs, paths.len());
    if job_count <= 1 {
        return run_tests_sequential(config, paths, standard_library, stdout, reporter);
    }

    run_tests_parallel(config, paths, standard_library, stdout, reporter)
}

/// Runs [`test_cli`] with diagnostics in the configured format written to `stderr`.
#[cfg(test)]
pub(crate) fn test_cli_with_stderr(
    config: TestCliConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let mut reporter = Reporter::new(config.diagnostics, stderr);
    test_cli(config, stdout, &mut reporter)
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
