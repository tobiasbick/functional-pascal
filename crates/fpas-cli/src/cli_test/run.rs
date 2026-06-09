//! Run a single FPAS test program.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::hooks::{TestHook, TestHooks, hook_program_source};
use super::report::TestOutcome;
use super::timeout::{VmRunResult, run_with_timeout};
use crate::cli_run::render_cli_diagnostic_with_sources;
use crate::test_script::{ScriptConfig, apply_script_to_vm, load_script, sidecar_path_for_test};
use fpas_diagnostics::DiagnosticSeverity;
use fpas_diagnostics::codes::RUNTIME_TEST_ASSERTION_FAILED;
use fpas_parser::parse;
use fpas_project as project;

fn test_display_path(path: &Path) -> std::borrow::Cow<'_, str> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(std::borrow::Cow::from)
        .unwrap_or_else(|| path.to_string_lossy())
}

/// Project sources used when linking a test program with local units.
pub(super) struct LinkContext {
    pub source_files: Vec<PathBuf>,
    pub link_meta: project::ProjectLinkMeta,
    pub test_manifest: project::TestManifest,
    pub hooks: TestHooks,
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

    if let Some(link) = link {
        if let Some(hook) = link.hooks.setup.as_ref() {
            let outcome = run_test_hook(hook, "Setup", path, link, stderr, &display);
            if outcome.is_failure() {
                let _ = run_optional_teardown(link, path, stderr, &display);
                return outcome;
            }
        }
    }

    let outcome = run_test_program(
        path,
        link,
        script_override,
        timeout,
        stderr,
        &display,
        RunOutput::Test,
    );

    if let Some(link) = link {
        if let Some(teardown_outcome) = run_optional_teardown(link, path, stderr, &display) {
            if outcome == TestOutcome::Pass && teardown_outcome.is_failure() {
                return teardown_outcome;
            }
        }
    }

    outcome
}

/// Controls PASS/FAIL lines emitted while executing a linked program.
enum RunOutput {
    Test,
    Hook,
}

impl RunOutput {
    fn emit_pass(self) -> bool {
        matches!(self, Self::Test)
    }

    fn emit_fail_banner(self) -> bool {
        matches!(self, Self::Test)
    }
}

fn run_optional_teardown(
    link: &LinkContext,
    path: &Path,
    stderr: &mut dyn Write,
    display: &str,
) -> Option<TestOutcome> {
    link.hooks
        .teardown
        .as_ref()
        .map(|hook| run_test_hook(hook, "Teardown", path, link, stderr, display))
}

fn run_test_hook(
    hook: &TestHook,
    label: &str,
    test_path: &Path,
    link: &LinkContext,
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
        None,
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

fn run_test_program(
    path: &Path,
    link: Option<&LinkContext>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
    stderr: &mut dyn Write,
    display: &str,
    output: RunOutput,
) -> TestOutcome {
    let path_text = path.to_string_lossy();
    let (program, source_paths) = match load_program(path, link) {
        Ok(value) => value,
        Err(message) => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            let _ = writeln!(stderr, "        {message}");
            return TestOutcome::CompileError;
        }
    };

    let chunk = match fpas_compiler::compile_all(&program) {
        Ok(chunk) => chunk,
        Err(diagnostics) => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            for diagnostic in &diagnostics {
                let _ = writeln!(
                    stderr,
                    "        {}",
                    render_cli_diagnostic_with_sources(
                        path_text.as_ref(),
                        source_paths.as_deref(),
                        diagnostic,
                    )
                    .replace('\n', "\n        ")
                );
            }
            return TestOutcome::CompileError;
        }
    };

    let mut vm = fpas_vm::Vm::new(chunk);
    let manifest_override = link.and_then(|ctx| ctx.test_manifest.override_for(path));
    let script_config = match apply_test_script(path, script_override, manifest_override, &mut vm) {
        Ok(config) => config,
        Err(message) => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            let _ = writeln!(stderr, "        {message}");
            return TestOutcome::CompileError;
        }
    };

    let headless_graph = script_config.headless_graph;
    let shutdown = vm.shutdown_handle();
    let run_result = if let Some(timeout) = timeout {
        run_with_timeout(shutdown, timeout, move || {
            if headless_graph {
                fpas_std::with_headless_graph_backend_for_tests(|| vm.run())
            } else {
                vm.run()
            }
        })
    } else if headless_graph {
        VmRunResult::Completed(fpas_std::with_headless_graph_backend_for_tests(|| vm.run()))
    } else {
        VmRunResult::Completed(vm.run())
    };

    match run_result {
        VmRunResult::TimedOut => {
            let seconds = timeout.map(|value| value.as_secs()).unwrap_or(0);
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  TIMEOUT  {display}");
            }
            let _ = writeln!(
                stderr,
                "        test run exceeded {seconds} second timeout.\n  help: Fix an infinite loop or increase `--timeout`."
            );
            TestOutcome::TimedOut
        }
        VmRunResult::Completed(Ok(())) => {
            if output.emit_pass() {
                let _ = writeln!(stderr, "  PASS  {display}");
            }
            TestOutcome::Pass
        }
        VmRunResult::Completed(Err(diagnostic)) => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            let _ = writeln!(
                stderr,
                "        {}",
                render_cli_diagnostic_with_sources(
                    path_text.as_ref(),
                    source_paths.as_deref(),
                    &diagnostic,
                )
                .replace('\n', "\n        ")
            );
            if diagnostic.code == RUNTIME_TEST_ASSERTION_FAILED {
                TestOutcome::AssertFailed
            } else {
                TestOutcome::RuntimeError
            }
        }
    }
}

fn write_temp_hook_program(test_path: &Path, source: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let file_name = format!(
        "fpas-test-hook-{}-{}.fpas",
        test_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("test"),
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

fn load_program(
    path: &Path,
    link: Option<&LinkContext>,
) -> Result<(fpas_parser::Program, Option<Vec<PathBuf>>), String> {
    if let Some(link) = link {
        let linked =
            project::build_program_with_source_map(path, &link.source_files, &link.link_meta)
                .map_err(|message| message)?;
        return Ok((linked.program, Some(linked.source_paths)));
    }

    let source = fs::read_to_string(path)
        .map_err(|error| format!("Error reading `{}`: {error}", path.display()))?;
    let (program, errors) = parse(&source);
    let has_errors = errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);
    if has_errors {
        return Err(format!(
            "Parse errors in `{}`.\n  help: Fix syntax errors before running tests.",
            path.display()
        ));
    }
    Ok((program, None))
}

fn apply_test_script(
    test_path: &Path,
    cli_script: Option<&Path>,
    manifest_override: Option<&project::TestFileOverride>,
    vm: &mut fpas_vm::Vm,
) -> Result<ScriptConfig, String> {
    let script_path = resolve_script_path(test_path, cli_script, manifest_override)?;

    let mut config = if let Some(script_path) = script_path {
        if !script_path.is_file() {
            return Err(format!(
                "Script file not found: `{}`.\n  help: Pass an existing `.script.toml` path with `--script` or fix `[test.overrides]` in the project file.",
                script_path.display()
            ));
        }

        let script = load_script(&script_path)?;
        apply_script_to_vm(vm, &script)?;
        script.config
    } else {
        ScriptConfig::default()
    };

    if let Some(manifest) = manifest_override {
        if let Some(headless_graph) = manifest.headless_graph {
            config.headless_graph = headless_graph;
        }
    }

    Ok(config)
}

fn resolve_script_path(
    test_path: &Path,
    cli_script: Option<&Path>,
    manifest_override: Option<&project::TestFileOverride>,
) -> Result<Option<PathBuf>, String> {
    if let Some(path) = cli_script {
        return Ok(Some(path.to_path_buf()));
    }

    if let Some(path) = manifest_override.and_then(|value| value.script.as_ref()) {
        return Ok(Some(path.clone()));
    }

    let sidecar = sidecar_path_for_test(test_path);
    if sidecar.is_file() {
        return Ok(Some(sidecar));
    }

    Ok(None)
}
