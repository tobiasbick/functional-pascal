//! Load and link a single FPAS test program before execution.

use std::fs;
use std::path::{Path, PathBuf};

use fpas_diagnostics::DiagnosticSeverity;
use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_project as project;

use crate::test_script::{apply_script_to_vm, load_script, sidecar_path_for_test};

use super::LinkContext;

pub(super) fn load_program(
    path: &Path,
) -> Result<(fpas_parser::Program, Option<Vec<PathBuf>>), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("Error reading `{}`: {error}", path.display()))?;
    let (unit, errors) = parse_compilation_unit(&source);
    let has_errors = errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error);
    if has_errors {
        return Err(format!(
            "Parse errors in `{}`.\n  help: Fix syntax errors before running tests.",
            path.display()
        ));
    }
    match unit {
        CompilationUnit::Program(program) => Ok((program, None)),
        CompilationUnit::Unit(unit) => {
            let unit_name = unit.name.parts.join(".").trim().to_string();
            Err(format!(
                "Test file `{}` declares `unit {unit_name}`, but test entry points must be `program` files.\n  help: Rename to a `program …_test` file or import the unit from a test program.",
                path.display()
            ))
        }
    }
}

pub(super) fn reject_unit_test_entry(path: &Path, link: &LinkContext) -> Result<(), String> {
    if !link.source_files.is_empty() {
        return Ok(());
    }

    let source = fs::read_to_string(path)
        .map_err(|error| format!("Error reading `{}`: {error}", path.display()))?;
    let (unit, errors) = parse_compilation_unit(&source);
    if errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error)
    {
        return Ok(());
    }
    let CompilationUnit::Unit(unit) = unit else {
        return Ok(());
    };
    let unit_name = unit.name.parts.join(".").trim().to_string();
    Err(format!(
        "Test file `{}` declares `unit {unit_name}`, but test entry points must be `program` files.\n  help: Rename to a `program …_test` file or import the unit from a test program.",
        path.display()
    ))
}

pub(super) fn apply_test_script(
    test_path: &Path,
    cli_script: Option<&Path>,
    manifest_override: Option<&project::TestFileOverride>,
    vm: &mut fpas_vm::Vm,
) -> Result<(), String> {
    let script_path = resolve_script_path(test_path, cli_script, manifest_override)?;

    if let Some(script_path) = script_path {
        if !script_path.is_file() {
            return Err(format!(
                "Script file not found: `{}`.\n  help: Pass an existing `.script.toml` path with `--script` or fix `[test.overrides]` in the project file.",
                script_path.display()
            ));
        }

        let script = load_script(&script_path)?;
        apply_script_to_vm(vm, &script);
    }
    Ok(())
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
