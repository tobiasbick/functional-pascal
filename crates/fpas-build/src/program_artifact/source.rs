//! Parsing of authoritative main-program source snapshots.

use fpas_parser::{CompilationUnit, Program};

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
