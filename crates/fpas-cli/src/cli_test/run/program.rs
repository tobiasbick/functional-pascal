//! Compile, execute, and classify one FPAS test program run.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fpas_diagnostics::codes::{RUNTIME_TEST_ASSERTION_FAILED, TEST_RUNNER_FAILED};
use fpas_vm::VmError;
use serde::{Deserialize, Serialize};

use super::super::expect_stdout;
use super::super::log;
use super::super::process;
use super::super::report::TestOutcome;
use super::LinkContext;
use crate::cli_output::{CliFailure, Reporter, locate};

use super::load::apply_test_script;

/// One test entry in a shared, memory-only bytecode image.
#[derive(Clone)]
pub(in crate::cli_test) struct CompiledTestProgram {
    pub image: Arc<fpas_bytecode::VerifiedExecutable>,
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
    /// Print captured standard output after a PASS line too.
    pub show_output: bool,
    /// Optional entry in a shared in-memory image.
    pub compiled: Option<&'a CompiledTestProgram>,
    /// Runner-owned directory shared by the test body and its hooks.
    pub scratch_dir: &'a Path,
}

/// Fully compiled test input transferable to an isolated worker process.
pub(in crate::cli_test) struct PreparedProgram {
    pub test_path: PathBuf,
    pub executable: fpas_bytecode::VerifiedExecutable,
    pub source_paths: Option<Arc<Vec<PathBuf>>>,
    pub script_override: Option<PathBuf>,
    pub manifest_override: Option<fpas_project::TestFileOverride>,
    pub display: String,
    pub output: RunOutput,
    pub show_output: bool,
    pub scratch_dir: PathBuf,
}

pub(super) fn run_test_program(
    path: &Path,
    link: Option<&LinkContext>,
    reporter: &mut Reporter<'_>,
    options: ProgramRunOptions<'_>,
) -> TestOutcome {
    let timeout = options.timeout;
    let prepared = match prepare_test_program(path, link, reporter, options) {
        Ok(prepared) => prepared,
        Err(outcome) => return outcome,
    };
    if let Some(timeout) = timeout {
        process::run_with_timeout(prepared, timeout, reporter)
    } else {
        run_prepared_program(prepared, reporter, || Ok(())).unwrap_or_else(|message| {
            log::message(
                reporter,
                TEST_RUNNER_FAILED,
                &format!("test worker failed unexpectedly: {message}"),
            );
            TestOutcome::RuntimeError
        })
    }
}

fn prepare_test_program(
    path: &Path,
    link: Option<&LinkContext>,
    reporter: &mut Reporter<'_>,
    options: ProgramRunOptions<'_>,
) -> Result<PreparedProgram, TestOutcome> {
    let ProgramRunOptions {
        script_override,
        timeout: _,
        display,
        output,
        show_output,
        compiled,
        scratch_dir,
    } = options;
    let built = if let Some(compiled) = compiled {
        Ok((
            (*compiled.image).clone(),
            Some(Arc::clone(&compiled.source_paths)),
        ))
    } else if let Some(link) = link {
        super::load::reject_unit_test_entry(path, link)
            .and_then(|()| {
                crate::project_build::build_test_program_with_graph(path, &link.program_graph)
            })
            .map(|built| (built.executable, Some(Arc::new(built.source_paths))))
    } else {
        super::load::load_program(path).and_then(|(program, source_paths)| {
            fpas_compiler::compile(&program)
                .map(|executable| (executable, source_paths.map(Arc::new)))
                .map_err(|diagnostics| CliFailure::from_diagnostics(path, &diagnostics))
        })
    };
    let (executable, source_paths) = match built {
        Ok(built) => built,
        Err(failure) => {
            render_failure(reporter, display, output, &failure);
            return Err(TestOutcome::CompileError);
        }
    };

    Ok(PreparedProgram {
        test_path: path.to_path_buf(),
        executable,
        source_paths,
        script_override: script_override.map(Path::to_path_buf),
        manifest_override: link
            .and_then(|context| context.test_manifest.override_for(path))
            .cloned(),
        display: display.to_string(),
        output,
        show_output,
        scratch_dir: scratch_dir.to_path_buf(),
    })
}

fn render_failure(
    reporter: &mut Reporter<'_>,
    display: &str,
    output: RunOutput,
    failure: &CliFailure,
) {
    if output.emit_fail_banner() {
        log::banner(reporter, "FAIL", display);
    }
    log::failure(reporter, failure);
}

/// Applies scripted input, opens the execution gate, runs the VM, and classifies its result.
pub(in crate::cli_test) fn run_prepared_program(
    prepared: PreparedProgram,
    reporter: &mut Reporter<'_>,
    gate: impl FnOnce() -> Result<(), String>,
) -> Result<TestOutcome, String> {
    let PreparedProgram {
        test_path,
        executable,
        source_paths,
        script_override,
        manifest_override,
        display,
        output,
        show_output,
        scratch_dir,
    } = prepared;
    // Runtime diagnostics carry linked source IDs; the prepared list is indexed by graph source.
    let source_paths =
        source_paths.map(|paths| fpas_build::linked_source_paths(&executable, &paths));
    let mut vm = fpas_vm::Vm::new(executable);
    vm.set_test_scratch_dir(scratch_dir);
    if let Err(failure) = apply_test_script(
        &test_path,
        script_override.as_deref(),
        manifest_override.as_ref(),
        &mut vm,
    ) {
        render_failure(reporter, &display, output, &failure);
        return Ok(TestOutcome::CompileError);
    }
    gate()?;
    let execution = execute_vm(vm);
    Ok(classify_execution(
        &test_path,
        source_paths.as_ref(),
        &display,
        (output, show_output),
        execution,
        reporter,
    ))
}

struct VmExecution {
    result: Result<(), VmError>,
    stdout_lines: Vec<String>,
    skipped: bool,
}

fn classify_execution(
    path: &Path,
    source_paths: Option<&Vec<PathBuf>>,
    display: &str,
    (output, show_output): (RunOutput, bool),
    execution: VmExecution,
    reporter: &mut Reporter<'_>,
) -> TestOutcome {
    match execution {
        VmExecution {
            result: Ok(()),
            ref stdout_lines,
            skipped,
        } => {
            if matches!(output, RunOutput::Test | RunOutput::TestDeferredPass)
                && let Err(failure) = expect_stdout::compare_stdout(path, stdout_lines)
            {
                render_failure(reporter, display, output, &failure);
                return TestOutcome::AssertFailed;
            }
            if skipped {
                if output.emit_pass() {
                    log::banner(reporter, "SKIP", display);
                }
                return TestOutcome::Skipped;
            }
            if output.emit_pass() {
                log::banner(reporter, "PASS", display);
                if show_output {
                    log::captured_stdout(reporter, stdout_lines);
                }
            }
            TestOutcome::Pass
        }
        VmExecution {
            result: Err(diagnostic),
            ref stdout_lines,
            skipped: _,
        } => {
            if output.emit_fail_banner() {
                log::banner(reporter, "FAIL", display);
            }
            log::record(
                reporter,
                &locate(path, source_paths.map(Vec::as_slice), &diagnostic),
            );
            log::captured_stdout(reporter, stdout_lines);
            if diagnostic.code == RUNTIME_TEST_ASSERTION_FAILED {
                TestOutcome::AssertFailed
            } else {
                TestOutcome::RuntimeError
            }
        }
    }
}

fn execute_vm(mut vm: fpas_vm::Vm) -> VmExecution {
    fpas_std::reset_test_skip_state();

    let result = vm.run().map(|_| ());
    VmExecution {
        result,
        stdout_lines: vm.output().lines,
        skipped: fpas_std::test_was_skipped(),
    }
}
