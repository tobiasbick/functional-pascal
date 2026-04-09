use fpas_diagnostics::{Diagnostic, DiagnosticSeverity};
use fpas_lexer::lex_with_source_id;
use fpas_parser::{CompilationUnit, QualifiedId, parse_tokens_compilation_unit};
use std::fs;
use std::path::Path;

pub(super) fn parse_compilation_unit_file(
    path: &Path,
    source_id: u32,
) -> Result<(CompilationUnit, Vec<String>), String> {
    let source_text = fs::read_to_string(path).map_err(|e| {
        format!(
            "Error reading source file `{}`: {e}",
            path.to_string_lossy()
        )
    })?;

    let (tokens, lex_errors) = lex_with_source_id(&source_text, source_id);
    let (unit, parse_errors) = parse_tokens_compilation_unit(tokens);

    let mut diagnostics: Vec<Diagnostic> = lex_errors;
    diagnostics.extend(
        parse_errors
            .into_iter()
            .map(|diagnostic| diagnostic.as_diagnostic().clone()),
    );

    let mut warnings = Vec::new();
    for diagnostic in diagnostics {
        if diagnostic.severity == DiagnosticSeverity::Error {
            let path_text = path.to_string_lossy();
            return Err(format!(
                "Failed to parse `{}`:\n  {}",
                path_text,
                fpas_diagnostics::render(path_text.as_ref(), &diagnostic)
            ));
        }

        warnings.push(fpas_diagnostics::render(
            path.to_string_lossy().as_ref(),
            &diagnostic,
        ));
    }

    Ok((unit, warnings))
}

pub(super) fn qualified_id_to_string(id: &QualifiedId) -> String {
    id.parts.join(".")
}

/// `docs/pascal/09-units.md`: `Std.*` is reserved for implementation-defined standard units.
pub(super) fn validate_user_unit_name(path: &Path, id: &QualifiedId) -> Result<(), String> {
    if id
        .parts
        .first()
        .is_some_and(|head| head.eq_ignore_ascii_case("std"))
    {
        return Err(format!(
            "Source file `{}` declares `unit {}`.\n  help: The root segment `Std` is reserved for standard library units. Rename the unit to a non-`Std` namespace such as `App.{}`.",
            path.to_string_lossy(),
            qualified_id_to_string(id),
            id.parts.get(1).map_or("Core", String::as_str)
        ));
    }

    Ok(())
}
