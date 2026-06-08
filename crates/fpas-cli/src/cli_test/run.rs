//! Run a single FPAS test program.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

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
    let path_text = path.to_string_lossy();
    let (program, source_paths) = match load_program(path, link) {
        Ok(value) => value,
        Err(message) => {
            let _ = writeln!(stderr, "  FAIL  {display}");
            let _ = writeln!(stderr, "        {message}");
            return TestOutcome::CompileError;
        }
    };

    let chunk = match fpas_compiler::compile_all(&program) {
        Ok(chunk) => chunk,
        Err(diagnostics) => {
            let _ = writeln!(stderr, "  FAIL  {display}");
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
    let script_config = match apply_test_script(path, script_override, &mut vm) {
        Ok(config) => config,
        Err(message) => {
            let _ = writeln!(stderr, "  FAIL  {display}");
            let _ = writeln!(stderr, "        {message}");
            return TestOutcome::CompileError;
        }
    };

    let headless_graph = script_config
        .as_ref()
        .is_some_and(|config| config.headless_graph);
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
            let _ = writeln!(stderr, "  TIMEOUT  {display}");
            let _ = writeln!(
                stderr,
                "        test run exceeded {seconds} second timeout.\n  help: Fix an infinite loop or increase `--timeout`."
            );
            TestOutcome::TimedOut
        }
        VmRunResult::Completed(Ok(())) => {
            let _ = writeln!(stderr, "  PASS  {display}");
            TestOutcome::Pass
        }
        VmRunResult::Completed(Err(diagnostic)) => {
            let _ = writeln!(stderr, "  FAIL  {display}");
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
    script_override: Option<&Path>,
    vm: &mut fpas_vm::Vm,
) -> Result<Option<ScriptConfig>, String> {
    let script_path = match script_override {
        Some(path) => path.to_path_buf(),
        None => {
            let sidecar = sidecar_path_for_test(test_path);
            if sidecar.is_file() {
                sidecar
            } else {
                return Ok(None);
            }
        }
    };

    if !script_path.is_file() {
        return Err(format!(
            "Script file not found: `{}`.\n  help: Pass an existing `.script.toml` path with `--script`.",
            script_path.display()
        ));
    }

    let script = load_script(&script_path)?;
    apply_script_to_vm(vm, &script)?;
    Ok(Some(script.config))
}
