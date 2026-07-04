//! Run a single FPAS test program.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

mod hook_exec;
mod load;
mod program;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fpas_project as project;

use super::hooks::TestHooks;
use super::report::TestOutcome;

use hook_exec::{run_optional_teardown, run_test_hook};
use program::{RunOutput, run_test_program};

/// Project sources used when linking a test program with local units.
#[derive(Clone)]
pub(super) struct LinkContext {
    pub source_files: Vec<PathBuf>,
    pub link_meta: project::ProjectLinkMeta,
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
pub(super) fn run_single_test_capture(
    path: &Path,
    link: Option<&LinkContext>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
) -> (TestOutcome, Vec<u8>) {
    let mut buffer = Vec::new();
    let outcome = run_single_test(path, link, script_override, timeout, &mut buffer);
    (outcome, buffer)
}

/// Compiles and runs one test file, classifying the result.
pub(super) fn run_single_test(
    path: &Path,
    link: Option<&LinkContext>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
    stderr: &mut dyn Write,
) -> TestOutcome {
    let display = test_display_path(path);
    let has_teardown = link.is_some_and(|context| context.hooks.teardown.is_some());

    if let Some(link) = link
        && let Some(hook) = link.hooks.setup.as_ref()
    {
        let outcome = run_test_hook(hook, "Setup", path, link, timeout, stderr, &display);
        if outcome.is_failure() {
            let _ = run_optional_teardown(link, path, timeout, stderr, &display);
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
        script_override,
        timeout,
        stderr,
        &display,
        body_output,
    );

    if let Some(link) = link
        && let Some(teardown_outcome) = run_optional_teardown(link, path, timeout, stderr, &display)
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
