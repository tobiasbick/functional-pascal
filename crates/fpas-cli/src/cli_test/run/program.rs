//! Compile, execute, and classify one FPAS test program run.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fpas_diagnostics::codes::RUNTIME_TEST_ASSERTION_FAILED;

use super::super::expect_pixels;
use super::super::expect_stdout;
use super::super::report::TestOutcome;
use super::super::timeout::{VmExecution, VmRunResult, run_with_timeout};
use super::LinkContext;
use crate::cli_run::render_cli_diagnostic_with_sources;

use super::load::{apply_test_script, load_program};

/// One test entry in a shared, memory-only bytecode image.
#[derive(Clone)]
pub(in crate::cli_test) struct CompiledTestProgram {
    pub image: Arc<fpas_bytecode::Chunk>,
    pub entry_ip: usize,
    pub source_paths: Arc<Vec<PathBuf>>,
}

/// Controls PASS/FAIL lines emitted while executing a linked program.
pub(super) enum RunOutput {
    /// Regular test body: prints PASS and FAIL banners.
    Test,
    /// Test body followed by a teardown hook: FAIL banners only; the caller
    /// prints PASS after the teardown hook also passed.
    TestDeferredPass,
    /// Setup/Teardown hook program: no banners; the hook wrapper reports failures.
    Hook,
}

impl RunOutput {
    fn emit_pass(&self) -> bool {
        matches!(self, Self::Test)
    }

    fn emit_fail_banner(&self) -> bool {
        matches!(self, Self::Test | Self::TestDeferredPass)
    }
}

/// Per-execution settings for one compiled or source-backed test program.
pub(super) struct ProgramRunOptions<'a> {
    /// Optional scripted input configuration.
    pub script_override: Option<&'a Path>,
    /// Optional wall-clock timeout.
    pub timeout: Option<Duration>,
    /// Test label used in progress and diagnostic output.
    pub display: &'a str,
    /// Controls PASS/FAIL banner emission.
    pub output: RunOutput,
    /// Optional entry in a shared in-memory image.
    pub compiled: Option<&'a CompiledTestProgram>,
}

pub(super) fn run_test_program(
    path: &Path,
    link: Option<&LinkContext>,
    stderr: &mut dyn Write,
    options: ProgramRunOptions<'_>,
) -> TestOutcome {
    let ProgramRunOptions {
        script_override,
        timeout,
        display,
        output,
        compiled,
    } = options;
    let path_text = path.to_string_lossy();
    let (mut vm, source_paths) = if let Some(compiled) = compiled {
        (
            fpas_vm::Vm::from_image(Arc::clone(&compiled.image), compiled.entry_ip),
            Some(Arc::clone(&compiled.source_paths)),
        )
    } else {
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
        (fpas_vm::Vm::new(chunk), source_paths.map(Arc::new))
    };
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
        run_with_timeout(shutdown, timeout, move || execute_vm(vm, headless_graph))
    } else {
        VmRunResult::Completed(execute_vm(vm, headless_graph))
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
        VmRunResult::WorkerFailed => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            let _ = writeln!(
                stderr,
                "        test worker failed unexpectedly.\n  help: Re-run under a debugger or report a compiler/runtime bug."
            );
            TestOutcome::RuntimeError
        }
        VmRunResult::Completed(VmExecution {
            result: Ok(()),
            ref stdout_lines,
            ref headless_frame,
            skipped,
        }) => {
            if matches!(output, RunOutput::Test | RunOutput::TestDeferredPass) {
                if let Err(message) = expect_stdout::compare_stdout(path, stdout_lines) {
                    if output.emit_fail_banner() {
                        let _ = writeln!(stderr, "  FAIL  {display}");
                    }
                    let _ = writeln!(stderr, "        {message}");
                    return TestOutcome::AssertFailed;
                }
                if let Some(frame) = headless_frame.as_ref()
                    && let Err(message) = expect_pixels::compare_pixels(path, frame)
                {
                    if output.emit_fail_banner() {
                        let _ = writeln!(stderr, "  FAIL  {display}");
                    }
                    let _ = writeln!(stderr, "        {message}");
                    return TestOutcome::AssertFailed;
                }
            }
            if skipped {
                if output.emit_pass() {
                    let _ = writeln!(stderr, "  SKIP  {display}");
                }
                return TestOutcome::Skipped;
            }
            if output.emit_pass() {
                let _ = writeln!(stderr, "  PASS  {display}");
            }
            TestOutcome::Pass
        }
        VmRunResult::Completed(VmExecution {
            result: Err(diagnostic),
            stdout_lines: _,
            headless_frame: _,
            skipped: _,
        }) => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            let _ = writeln!(
                stderr,
                "        {}",
                render_cli_diagnostic_with_sources(
                    path_text.as_ref(),
                    source_paths.as_deref().map(Vec::as_slice),
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

fn execute_vm(mut vm: fpas_vm::Vm, headless_graph: bool) -> VmExecution {
    fpas_std::reset_test_skip_state();

    let mut run = || {
        let result = vm.run();
        VmExecution {
            result,
            stdout_lines: vm.output().lines,
            headless_frame: fpas_std::last_headless_graph_frame_for_tests(),
            skipped: fpas_std::test_was_skipped(),
        }
    };

    if headless_graph {
        fpas_std::with_headless_graph_backend_for_tests(run)
    } else {
        run()
    }
}
