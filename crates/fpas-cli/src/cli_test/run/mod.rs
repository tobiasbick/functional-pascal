//! Run a single FPAS test program.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

mod hook_exec;
mod load;
pub(in crate::cli_test) mod program;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fpas_project as project;

use super::hooks::TestHooks;
use super::report::TestOutcome;
use super::scratch::TestScratch;

use hook_exec::{HookRunContext, run_optional_teardown, run_test_hook};
pub(super) use program::CompiledTestProgram;
use program::{ProgramRunOptions, RunOutput, run_test_program};

/// Project sources used when linking a test program with local units.
#[derive(Clone)]
pub(super) struct LinkContext {
    pub source_files: Vec<PathBuf>,
    pub program_graph: Arc<project::ProgramUnitGraph>,
    pub test_manifest: project::TestManifest,
    pub hooks: TestHooks,
}

pub(super) fn test_display_path(path: &Path) -> std::borrow::Cow<'_, str> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(std::borrow::Cow::from)
        .unwrap_or_else(|| path.to_string_lossy())
}

/// Compiles and runs one test file, capturing stderr-style output for parallel runs.
#[cfg(test)]
pub(super) fn run_single_test_capture(
    path: &Path,
    link: Option<&LinkContext>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
) -> (TestOutcome, Vec<u8>) {
    run_single_test_capture_prepared(path, link, script_override, timeout, None)
}

pub(super) fn run_single_test_capture_prepared(
    path: &Path,
    link: Option<&LinkContext>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
    compiled: Option<&CompiledTestProgram>,
) -> (TestOutcome, Vec<u8>) {
    let mut buffer = Vec::new();
    let outcome =
        run_single_test_prepared(path, link, script_override, timeout, &mut buffer, compiled);
    (outcome, buffer)
}

pub(super) fn run_single_test_prepared(
    path: &Path,
    link: Option<&LinkContext>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
    stderr: &mut dyn Write,
    compiled: Option<&CompiledTestProgram>,
) -> TestOutcome {
    let display = test_display_path(path);
    let scratch = match TestScratch::create(path) {
        Ok(scratch) => scratch,
        Err(message) => {
            let _ = writeln!(stderr, "  FAIL  {display}");
            let _ = writeln!(stderr, "        {message}");
            return TestOutcome::RuntimeError;
        }
    };
    let has_teardown = link.is_some_and(|context| context.hooks.teardown.is_some());

    if let Some(link) = link
        && let Some(hook) = link.hooks.setup.as_ref()
    {
        let outcome = run_test_hook(
            hook,
            "Setup",
            HookRunContext {
                test_path: path,
                link,
                timeout,
                display: &display,
                scratch_dir: scratch.path(),
            },
            stderr,
        );
        if outcome.is_failure() {
            let _ = run_optional_teardown(link, path, timeout, stderr, &display, scratch.path());
            return outcome;
        }
    }

    // With a teardown hook present, the PASS line is deferred until the hook
    // also succeeded so the log never shows PASS followed by FAIL for one test.
    let body_output = if has_teardown {
        RunOutput::TestDeferredPass
    } else {
        RunOutput::Test
    };
    let outcome = run_test_program(
        path,
        link,
        stderr,
        ProgramRunOptions {
            script_override,
            timeout,
            display: &display,
            output: body_output,
            compiled,
            scratch_dir: scratch.path(),
        },
    );

    if let Some(link) = link
        && let Some(teardown_outcome) =
            run_optional_teardown(link, path, timeout, stderr, &display, scratch.path())
        && outcome == TestOutcome::Pass
        && teardown_outcome.is_failure()
    {
        return teardown_outcome;
    }

    if has_teardown && outcome == TestOutcome::Pass {
        let _ = writeln!(stderr, "  PASS  {display}");
    }

    outcome
}
