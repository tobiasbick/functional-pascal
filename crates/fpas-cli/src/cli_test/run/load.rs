//! Load and link a single FPAS test program before execution.
//!
//! Failures are coded records; see `docs/pascal/tools/diagnostics.md`.

use std::fs;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::{PROJECT_UNIT_KIND_MISMATCH, TEST_RUNNER_FAILED};
use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_project as project;

use crate::cli_output::CliFailure;
use crate::project_build::source_read_failure;
use crate::test_script::{apply_script_to_vm, load_script, sidecar_path_for_test};

use super::LinkContext;

pub(super) fn load_program(
    path: &Path,
) -> Result<(fpas_parser::Program, Option<Vec<PathBuf>>), CliFailure> {
    let source = fs::read_to_string(path).map_err(|error| source_read_failure(path, &error))?;
    let (unit, diagnostics) = parse_compilation_unit(&source);
    let errors = diagnostics
        .iter()
        .map(fpas_parser::ParseDiagnostic::as_diagnostic)
        .filter(|diagnostic| diagnostic.is_error())
        .cloned()
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(CliFailure::from_diagnostics(path, &errors));
    }
    match unit {
        CompilationUnit::Program(program) => Ok((program, None)),
        CompilationUnit::Unit(unit) => Err(unit_entry_failure(path, &unit)),
    }
}

pub(super) fn reject_unit_test_entry(path: &Path, link: &LinkContext) -> Result<(), CliFailure> {
    if !link.source_files.is_empty() {
        return Ok(());
    }

    let source = fs::read_to_string(path).map_err(|error| source_read_failure(path, &error))?;
    let (unit, errors) = parse_compilation_unit(&source);
    if errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().is_error())
    {
        return Ok(());
    }
    match unit {
        CompilationUnit::Unit(unit) => Err(unit_entry_failure(path, &unit)),
        CompilationUnit::Program(_) => Ok(()),
    }
}

fn unit_entry_failure(path: &Path, unit: &fpas_parser::Unit) -> CliFailure {
    let unit_name = unit.name.parts.join(".").trim().to_string();
    CliFailure::new(
        PROJECT_UNIT_KIND_MISMATCH,
        format!(
            "Test file declares `unit {unit_name}`, but test entry points must be `program` files."
        ),
    )
    .with_help("Rename to a `program …_test` file or import the unit from a test program.")
    .in_file(path)
}

pub(super) fn apply_test_script(
    test_path: &Path,
    cli_script: Option<&Path>,
    manifest_override: Option<&project::TestFileOverride>,
    vm: &mut fpas_vm::Vm,
) -> Result<(), CliFailure> {
    let script_path = resolve_script_path(test_path, cli_script, manifest_override);

    if let Some(script_path) = script_path {
        if !script_path.is_file() {
            return Err(CliFailure::new(
                TEST_RUNNER_FAILED,
                format!("Script file not found: `{}`.", script_path.display()),
            )
            .with_help("Pass an existing `.script.toml` path with `--script` or fix `[test.overrides]` in the project file."));
        }

        let script = load_script(&script_path)
            .map_err(|message| CliFailure::from_message(TEST_RUNNER_FAILED, &message))?;
        apply_script_to_vm(vm, &script);
    }
    Ok(())
}

fn resolve_script_path(
    test_path: &Path,
    cli_script: Option<&Path>,
    manifest_override: Option<&project::TestFileOverride>,
) -> Option<PathBuf> {
    if let Some(path) = cli_script {
        return Some(path.to_path_buf());
    }

    if let Some(path) = manifest_override.and_then(|value| value.script.as_ref()) {
        return Some(path.clone());
    }

    let sidecar = sidecar_path_for_test(test_path);
    sidecar.is_file().then_some(sidecar)
}
