//! `fpas test` — discover and run `*_test.fpas` programs.
//!
//! Spec: [`docs/pascal/program-structure/cli.md`](../../../docs/pascal/program-structure/cli.md),
//! [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md).

mod discover;
mod expect_stdout;
mod hooks;
mod image;
mod link;
mod parallel;
mod process;
mod report;
mod run;
mod runner;

#[cfg(test)]
mod tests;

use std::io::Write;

use crate::cli_input::TestCliConfig;
use discover::{discover_test_files, filter_test_paths};
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
    stderr: &mut dyn Write,
) -> i32 {
    let standard_library =
        match crate::standard_library::resolve_standard_library(config.standard_library.as_deref())
        {
            Ok(library) => library.map(Arc::new),
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
        };
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
            if let Err(exit_code) = crate::cli_output::write_stdout(
                stdout,
                stderr,
                "test file list to stdout",
                |stdout| writeln!(stdout, "{}", path.display()),
            ) {
                return exit_code;
            }
        }
        return 0;
    }

    let _ = writeln!(stderr, "Running {} test(s)...", paths.len());
    let _ = writeln!(stderr);

    let job_count = effective_job_count(config.jobs, paths.len());
    if job_count <= 1 {
        return run_tests_sequential(config, paths, standard_library, stdout, stderr);
    }

    run_tests_parallel(config, paths, standard_library, stdout, stderr)
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
