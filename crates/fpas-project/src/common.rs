use fpas_diagnostics::Diagnostic;
use fpas_lexer::lex_with_source_id;
use fpas_parser::{CompilationUnit, QualifiedId, parse_tokens_compilation_unit};
use std::fs;
use std::path::Path;

/// Pascal-cases a lowercase dotted unit key for diagnostics (`mylib.core` → `Mylib.Core`).
pub(super) fn display_unit_key(key: &str) -> String {
    let mut result = String::new();
    for (index, segment) in key.split('.').enumerate() {
        if index > 0 {
            result.push('.');
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }
    result
}

pub(super) fn validate_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "`{field_name}` must be a non-empty string.\n  help: Provide a value such as `\"my-app\"`."
        ));
    }
    Ok(())
}

pub(super) fn validate_non_empty_entry(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "A `{field_name}` entry is empty.\n  help: Remove empty entries or provide a valid value."
        ));
    }
    Ok(())
}

pub(super) fn parse_compilation_unit_file(
    path: &Path,
    source_id: u32,
) -> Result<(CompilationUnit, Vec<String>), String> {
    let source = fs::read(path).map_err(|e| {
        format!(
            "Error reading source file `{}`: {e}",
            path.to_string_lossy()
        )
    })?;
    parse_compilation_unit_source(path, &source, source_id)
}

pub(super) fn read_compilation_unit_file(
    path: &Path,
    source_id: u32,
) -> Result<(Vec<u8>, CompilationUnit, Vec<String>), String> {
    let source = fs::read(path).map_err(|e| {
        format!(
            "Error reading source file `{}`: {e}",
            path.to_string_lossy()
        )
    })?;
    let (unit, warnings) = parse_compilation_unit_source(path, &source, source_id)?;
    Ok((source, unit, warnings))
}

pub(super) fn parse_compilation_unit_source(
    path: &Path,
    source: &[u8],
    source_id: u32,
) -> Result<(CompilationUnit, Vec<String>), String> {
    let source_text = std::str::from_utf8(source).map_err(|error| {
        format!(
            "Source file `{}` is not valid UTF-8: {error}",
            path.to_string_lossy()
        )
    })?;

    let (tokens, _comments, lex_errors) = lex_with_source_id(source_text, source_id);
    let (unit, parse_errors) = parse_tokens_compilation_unit(tokens);

    let mut diagnostics: Vec<Diagnostic> = lex_errors;
    diagnostics.extend(
        parse_errors
            .into_iter()
            .map(|diagnostic| diagnostic.as_diagnostic().clone()),
    );

    let mut warnings = Vec::new();
    for diagnostic in diagnostics {
        if diagnostic.is_error() {
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

pub(super) enum SourceHeader {
    Program(String),
    Unit(QualifiedId),
}

pub(super) fn read_source_header(
    path: &Path,
    source_id: u32,
) -> Result<(SourceHeader, Vec<String>), String> {
    if let Ok(source) = fs::read(path)
        && let Ok(sidecar) = fs::read(fpas_unit::sidecar_path(path))
        && let Ok(compiled) = fpas_unit::decode(&sidecar)
        && compiled.identity.source_hash == fpas_unit::Digest::of(&source)
    {
        return Ok((
            SourceHeader::Unit(QualifiedId {
                parts: compiled
                    .identity
                    .unit_name
                    .split('.')
                    .map(str::to_string)
                    .collect(),
                span: fpas_lexer::Span {
                    offset: 0,
                    length: 0,
                    line: 1,
                    column: 1,
                    source_id,
                },
            }),
            Vec::new(),
        ));
    }

    let (unit, warnings) = parse_compilation_unit_file(path, source_id)?;
    let header = match unit {
        CompilationUnit::Program(program) => SourceHeader::Program(program.name),
        CompilationUnit::Unit(unit) => SourceHeader::Unit(unit.name),
    };
    Ok((header, warnings))
}

pub(super) fn qualified_id_to_string(id: &QualifiedId) -> String {
    id.parts.join(".")
}

/// `docs/pascal/program-structure/units.md`: `Std.*` is reserved for implementation-defined standard units.
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
