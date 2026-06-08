//! `fpas test` — discover and run `*_test.fpas` programs.
//!
//! Spec: [`docs/pascal/10-projects.md`](../../../docs/pascal/10-projects.md),
//! [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md).

mod discover;
mod report;
mod run;

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_input::TestCliConfig;
use discover::{discover_test_files, is_test_file_name};
use fpas_project as project;
use report::{Summary, print_summary};
use run::{LinkContext, run_single_test};

/// Runs discovered tests and prints a pass/fail summary.
pub(crate) fn test_cli(config: TestCliConfig, stderr: &mut dyn Write) -> i32 {
    let paths = match discover_test_files(&config.input, config.cwd.as_path()) {
        Ok(paths) => paths,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 2;
        }
    };

    if paths.is_empty() {
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
        let display = path.display().to_string();
        let link = link_context_for_test(&path);
        let outcome = run_single_test(&path, link.as_ref(), stderr);
        if config.fail_fast && outcome.is_failure() {
            summary.record(&display, outcome);
            let _ = print_summary(stderr, &summary);
            return summary.exit_code();
        }
        summary.record(&display, outcome);
    }

    let _ = print_summary(stderr, &summary);
    summary.exit_code()
}

fn link_context_for_test(path: &Path) -> Option<LinkContext> {
    let project_file = find_enclosing_project(path)?;
    let loaded = project::load_project(&project_file).ok()?;
    Some(LinkContext {
        source_files: loaded.source_files,
        link_meta: loaded.link_meta,
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

        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.clone()),
                cwd: cwd.clone(),
                fail_fast: false,
                list_only: false,
            },
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

        let mut stderr = Vec::new();
        let exit = test_cli(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.clone()),
                cwd,
                fail_fast: false,
                list_only: true,
            },
            &mut stderr,
        );

        assert_eq!(exit, 0);
        let text = String::from_utf8(stderr).expect("utf-8");
        assert!(text.contains("one_test.fpas"));
        assert!(!text.contains("FAIL"));
    }
}
