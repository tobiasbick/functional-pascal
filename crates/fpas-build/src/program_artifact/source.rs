//! Parsing and validation of authoritative program source snapshots.

use std::fs;
use std::path::Path;

use fpas_diagnostics::Diagnostic;
use fpas_diagnostics::codes::{
    INTERNAL_PROJECT_INVARIANT_FAILURE, PROJECT_SOURCE_INVALID_UTF8, PROJECT_UNIT_KIND_MISMATCH,
};
use fpas_parser::{CompilationUnit, Program};
use fpas_program::Digest;
use fpas_project::{UnitGraph, UnitNode};

use crate::BuildError;
use crate::source_snapshot::{source_changed_error, source_read_error};

pub(super) fn parse(bytes: &[u8], source_paths: &[String]) -> Result<Program, BuildError> {
    let path = source_paths.first().map(Path::new);
    let source = std::str::from_utf8(bytes).map_err(|error| {
        attributed(
            BuildError::new(
                PROJECT_SOURCE_INVALID_UTF8,
                format!("Source file is not valid UTF-8: {error}"),
            )
            .with_help("Save the source file as UTF-8 and retry."),
            path,
        )
    })?;
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(source);
    if diagnostics
        .iter()
        .map(fpas_parser::ParseDiagnostic::as_diagnostic)
        .any(Diagnostic::is_error)
    {
        return Err(BuildError::from_diagnostics(
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.as_diagnostic().clone())
                .collect(),
            path.map(|path| (0, path)),
        ));
    }
    match unit {
        CompilationUnit::Program(program) => Ok(program),
        CompilationUnit::Unit(unit) => Err(attributed(
            BuildError::new(
                PROJECT_UNIT_KIND_MISMATCH,
                format!(
                    "Main source declares unit `{}` instead of a program.",
                    unit.name.parts.join(".")
                ),
            )
            .with_help("Use a `program` declaration in the main file."),
            path,
        )),
    }
}

/// Rejects a program snapshot when its main source or any graph unit changed on disk.
pub(super) fn ensure_current(graph: &UnitGraph, main_hash: Digest) -> Result<(), BuildError> {
    let main_path = graph.source_paths().first().ok_or_else(|| {
        BuildError::new(
            INTERNAL_PROJECT_INVARIANT_FAILURE,
            "cannot validate a program snapshot without its main source path",
        )
    })?;
    ensure_path_current(main_path, main_hash)?;
    for (_, node) in graph.iter() {
        ensure_unit_current(node)?;
    }
    Ok(())
}

fn ensure_unit_current(node: &UnitNode) -> Result<(), BuildError> {
    let expected = node.source_hash().ok_or_else(|| {
        BuildError::new(
            INTERNAL_PROJECT_INVARIANT_FAILURE,
            format!(
                "cannot validate unit `{}` without an authoritative source snapshot",
                node.display_name()
            ),
        )
    })?;
    ensure_path_current(node.path(), expected)
}

fn ensure_path_current(path: &Path, expected: Digest) -> Result<(), BuildError> {
    let bytes = fs::read(path).map_err(|error| source_read_error(path, &error))?;
    if Digest::of(bytes) != expected {
        return Err(source_changed_error(path, "during the build"));
    }
    Ok(())
}

fn attributed(error: BuildError, path: Option<&Path>) -> BuildError {
    match path {
        Some(path) => error.in_source(path),
        None => error,
    }
}
