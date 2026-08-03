//! Compile, execute, and classify one FPAS test program run.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fpas_diagnostics::codes::RUNTIME_TEST_ASSERTION_FAILED;
use fpas_std::UploadedFrame;
use fpas_vm::VmError;
use serde::{Deserialize, Serialize};

use super::super::expect_pixels;
use super::super::expect_stdout;
use super::super::process;
use super::super::report::TestOutcome;
use super::LinkContext;
use crate::cli_run::render_cli_diagnostic_with_sources;

use super::load::{apply_test_script, load_program};

/// One test entry in a shared, memory-only bytecode image.
#[derive(Clone)]
pub(in crate::cli_test) struct CompiledTestProgram {
    pub image: Arc<fpas_bytecode::Chunk>,
    pub source_paths: Arc<Vec<PathBuf>>,
}

/// Controls PASS/FAIL lines emitted while executing a linked program.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(in crate::cli_test) enum RunOutput {
    /// Regular test body: prints PASS and FAIL banners.
    Test,
    /// Test body followed by a teardown hook: FAIL banners only; the caller
    /// prints PASS after the teardown hook also passed.
    TestDeferredPass,
    /// Setup/Teardown hook program: no banners; the hook wrapper reports failures.
    Hook,
}

impl RunOutput {
    pub(in crate::cli_test) fn emit_pass(self) -> bool {
        matches!(self, Self::Test)
    }

    pub(in crate::cli_test) fn emit_fail_banner(self) -> bool {
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

/// Fully compiled test input transferable to an isolated worker process.
pub(in crate::cli_test) struct PreparedProgram {
    pub test_path: PathBuf,
    pub chunk: fpas_bytecode::Chunk,
    pub source_paths: Option<Arc<Vec<PathBuf>>>,
    pub script_override: Option<PathBuf>,
    pub manifest_override: Option<fpas_project::TestFileOverride>,
    pub display: String,
    pub output: RunOutput,
}

pub(super) fn run_test_program(
    path: &Path,
    link: Option<&LinkContext>,
    stderr: &mut dyn Write,
    options: ProgramRunOptions<'_>,
) -> TestOutcome {
    let timeout = options.timeout;
    let prepared = match prepare_test_program(path, link, stderr, options) {
        Ok(prepared) => prepared,
        Err(outcome) => return outcome,
    };
    if let Some(timeout) = timeout {
        process::run_with_timeout(prepared, timeout, stderr)
    } else {
        run_prepared_program(prepared, stderr, || Ok(())).unwrap_or_else(|message| {
            let _ = writeln!(stderr, "        test worker failed unexpectedly: {message}");
            TestOutcome::RuntimeError
        })
    }
}

fn prepare_test_program(
    path: &Path,
    link: Option<&LinkContext>,
    stderr: &mut dyn Write,
    options: ProgramRunOptions<'_>,
) -> Result<PreparedProgram, TestOutcome> {
    let ProgramRunOptions {
        script_override,
        timeout: _,
        display,
        output,
        compiled,
    } = options;
    let path_text = path.to_string_lossy();
    let (chunk, source_paths) = if let Some(compiled) = compiled {
        (
            (*compiled.image).clone(),
            Some(Arc::clone(&compiled.source_paths)),
        )
    } else if let Some(link) = link {
        if let Err(message) = super::load::reject_unit_test_entry(path, link) {
            render_compile_error(stderr, display, output, &message);
            return Err(TestOutcome::CompileError);
        }
        let built = match crate::project_build::build_test_program(
            path,
            &link.source_files,
            &link.link_meta,
            link.standard_library.as_deref(),
        ) {
            Ok(built) => built,
            Err(message) => {
                render_compile_error(stderr, display, output, &message);
                return Err(TestOutcome::CompileError);
            }
        };
        (built.chunk, Some(Arc::new(built.source_paths)))
    } else {
        let (program, source_paths) = match load_program(path) {
            Ok(value) => value,
            Err(message) => {
                render_compile_error(stderr, display, output, &message);
                return Err(TestOutcome::CompileError);
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
                return Err(TestOutcome::CompileError);
            }
        };
        (chunk, source_paths.map(Arc::new))
    };

    Ok(PreparedProgram {
        test_path: path.to_path_buf(),
        chunk,
        source_paths,
        script_override: script_override.map(Path::to_path_buf),
        manifest_override: link
            .and_then(|context| context.test_manifest.override_for(path))
            .cloned(),
        display: display.to_string(),
        output,
    })
}

fn render_compile_error(stderr: &mut dyn Write, display: &str, output: RunOutput, message: &str) {
    if output.emit_fail_banner() {
        let _ = writeln!(stderr, "  FAIL  {display}");
    }
    let _ = writeln!(stderr, "        {message}");
}

/// Applies scripted input, opens the execution gate, runs the VM, and classifies its result.
pub(in crate::cli_test) fn run_prepared_program(
    prepared: PreparedProgram,
    stderr: &mut dyn Write,
    gate: impl FnOnce() -> Result<(), String>,
) -> Result<TestOutcome, String> {
    let PreparedProgram {
        test_path,
        chunk,
        source_paths,
        script_override,
        manifest_override,
        display,
        output,
    } = prepared;
    let mut vm = fpas_vm::Vm::new(chunk);
    let script_config = match apply_test_script(
        &test_path,
        script_override.as_deref(),
        manifest_override.as_ref(),
        &mut vm,
    ) {
        Ok(config) => config,
        Err(message) => {
            render_compile_error(stderr, &display, output, &message);
            return Ok(TestOutcome::CompileError);
        }
    };
    gate()?;
    let execution = execute_vm(vm, script_config.headless_graph);
    Ok(classify_execution(
        &test_path,
        source_paths.as_deref(),
        &display,
        output,
        execution,
        stderr,
    ))
}

struct VmExecution {
    result: Result<(), VmError>,
    stdout_lines: Vec<String>,
    headless_frame: Option<UploadedFrame>,
    skipped: bool,
}

fn classify_execution(
    path: &Path,
    source_paths: Option<&Vec<PathBuf>>,
    display: &str,
    output: RunOutput,
    execution: VmExecution,
    stderr: &mut dyn Write,
) -> TestOutcome {
    match execution {
        VmExecution {
            result: Ok(()),
            ref stdout_lines,
            ref headless_frame,
            skipped,
        } => {
            if matches!(output, RunOutput::Test | RunOutput::TestDeferredPass) {
                if let Err(message) = expect_stdout::compare_stdout(path, stdout_lines) {
                    render_assertion_error(stderr, display, output, &message);
                    return TestOutcome::AssertFailed;
                }
                if let Some(frame) = headless_frame.as_ref()
                    && let Err(message) = expect_pixels::compare_pixels(path, frame)
                {
                    render_assertion_error(stderr, display, output, &message);
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
        VmExecution {
            result: Err(diagnostic),
            stdout_lines: _,
            headless_frame: _,
            skipped: _,
        } => {
            if output.emit_fail_banner() {
                let _ = writeln!(stderr, "  FAIL  {display}");
            }
            let path_text = path.to_string_lossy();
            let _ = writeln!(
                stderr,
                "        {}",
                render_cli_diagnostic_with_sources(
                    path_text.as_ref(),
                    source_paths.map(Vec::as_slice),
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

fn render_assertion_error(stderr: &mut dyn Write, display: &str, output: RunOutput, message: &str) {
    if output.emit_fail_banner() {
        let _ = writeln!(stderr, "  FAIL  {display}");
    }
    let _ = writeln!(stderr, "        {message}");
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
