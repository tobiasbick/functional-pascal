//! Parsing and validation of authoritative program source snapshots.

use std::fs;

use fpas_parser::{CompilationUnit, Program};
use fpas_program::Digest;
use fpas_project::{UnitGraph, UnitNode};

use crate::BuildError;

pub(super) fn parse(bytes: &[u8], source_paths: &[String]) -> Result<Program, BuildError> {
    let path = source_paths.first().map_or("<program>", String::as_str);
    let source = std::str::from_utf8(bytes).map_err(|error| {
        BuildError::new(format!(
            "program source `{path}` is not valid UTF-8: {error}"
        ))
    })?;
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(source);
    if let Some(diagnostic) = diagnostics
        .iter()
        .map(fpas_parser::ParseDiagnostic::as_diagnostic)
        .find(|diagnostic| diagnostic.is_error())
    {
        return Err(BuildError::new(fpas_diagnostics::render(path, diagnostic)));
    }
    match unit {
        CompilationUnit::Program(program) => Ok(program),
        CompilationUnit::Unit(unit) => Err(BuildError::new(format!(
            "main source `{path}` declares unit `{}` instead of a program",
            unit.name.parts.join(".")
        ))),
    }
}

/// Rejects a program snapshot when its main source or any graph unit changed on disk.
pub(super) fn ensure_current(graph: &UnitGraph, main_hash: Digest) -> Result<(), BuildError> {
    let main_path = graph.source_paths().first().ok_or_else(|| {
        BuildError::new("cannot validate a program snapshot without its main source path")
    })?;
    ensure_path_current(main_path, main_hash)?;
    for (_, node) in graph.iter() {
        ensure_unit_current(node)?;
    }
    Ok(())
}

fn ensure_unit_current(node: &UnitNode) -> Result<(), BuildError> {
    let expected = node.source_hash().ok_or_else(|| {
        BuildError::new(format!(
            "cannot validate unit `{}` without an authoritative source snapshot",
            node.display_name()
        ))
    })?;
    ensure_path_current(node.path(), Digest::from_bytes(*expected.as_bytes()))
}

fn ensure_path_current(path: &std::path::Path, expected: Digest) -> Result<(), BuildError> {
    let bytes = fs::read(path).map_err(|error| {
        BuildError::new(format!(
            "cannot read program source `{}`: {error}",
            path.display()
        ))
    })?;
    if Digest::of(bytes) != expected {
        return Err(BuildError::new(format!(
            "program source `{}` changed during the build\n  help: Reload the project and retry the build.",
            path.display()
        )));
    }
    Ok(())
}
