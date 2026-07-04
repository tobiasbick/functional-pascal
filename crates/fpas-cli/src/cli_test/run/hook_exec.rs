//! Setup and teardown hook execution for linked test projects.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::super::hooks::{TestHook, hook_program_source};
use super::super::report::TestOutcome;
use super::LinkContext;
use super::program::{RunOutput, run_test_program};

pub(super) fn run_optional_teardown(
    link: &LinkContext,
    path: &Path,
    timeout: Option<Duration>,
    stderr: &mut dyn Write,
    display: &str,
) -> Option<TestOutcome> {
    link.hooks
        .teardown
        .as_ref()
        .map(|hook| run_test_hook(hook, "Teardown", path, link, timeout, stderr, display))
}

pub(super) fn run_test_hook(
    hook: &TestHook,
    label: &str,
    test_path: &Path,
    link: &LinkContext,
    timeout: Option<Duration>,
    stderr: &mut dyn Write,
    display: &str,
) -> TestOutcome {
    let hook_path = match write_temp_hook_program(test_path, &hook_program_source(hook)) {
        Ok(path) => path,
        Err(message) => {
            let _ = writeln!(stderr, "  FAIL  {display}");
            let _ = writeln!(stderr, "        {label} hook failed: {message}");
            return TestOutcome::CompileError;
        }
    };

    let outcome = run_test_program(
        &hook_path,
        Some(link),
        None,
        timeout,
        stderr,
        display,
        RunOutput::Hook,
    );
    let _ = fs::remove_file(&hook_path);

    if outcome.is_failure() {
        let _ = writeln!(stderr, "  FAIL  {display}");
        let _ = writeln!(
            stderr,
            "        {label} hook failed.\n  help: Fix the `{label}` procedure in the test project helper unit."
        );
    }
    outcome
}

fn write_temp_hook_program(test_path: &Path, source: &str) -> Result<PathBuf, String> {
    // Process id plus an atomic counter keeps names unique across parallel `--jobs`
    // workers and concurrent `fpas test` processes; a timestamp alone can collide.
    static NEXT_HOOK_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let unique = NEXT_HOOK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let dir = std::env::temp_dir();
    let file_name = format!(
        "fpas-test-hook-{}-{}-{}.fpas",
        test_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("test"),
        std::process::id(),
        unique
    );
    let path = dir.join(file_name);
    fs::write(&path, source).map_err(|error| {
        format!(
            "Error writing temporary hook program `{}`: {error}",
            path.display()
        )
    })?;
    Ok(path)
}
