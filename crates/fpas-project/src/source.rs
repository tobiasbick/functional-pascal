//! Source reading, unit-name display, and manifest string validation.

use fpas_diagnostics::codes::{
    PROJECT_DUPLICATE_UNIT, PROJECT_MANIFEST_VALUE_INVALID, PROJECT_PROGRAM_SOURCE_SKIPPED,
    PROJECT_UNIT_NAMESPACE_INVALID,
};
use fpas_diagnostics::{Diagnostic, FileDiagnostic};
use fpas_lexer::lex_with_source_id;
use fpas_parser::{CompilationUnit, QualifiedId, parse_tokens_compilation_unit};
mod error;
mod read;
use std::path::Path;

pub use error::ProjectError;

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

pub(super) fn validate_non_empty(field_name: &str, value: &str) -> Result<(), ProjectError> {
    if value.trim().is_empty() {
        return Err(ProjectError::new(
            PROJECT_MANIFEST_VALUE_INVALID,
            format!("`{field_name}` must be a non-empty string."),
        )
        .with_help("Provide a value such as `\"my-app\"`."));
    }
    Ok(())
}

pub(super) fn validate_non_empty_entry(field_name: &str, value: &str) -> Result<(), ProjectError> {
    if value.trim().is_empty() {
        return Err(ProjectError::new(
            PROJECT_MANIFEST_VALUE_INVALID,
            format!("A `{field_name}` entry is empty."),
        )
        .with_help("Remove empty entries or provide a valid value."));
    }
    Ok(())
}

/// Reads and parses one file without rendering source failures.
pub(super) fn parse_compilation_unit_file(
    path: &Path,
    source_id: u32,
) -> Result<(CompilationUnit, Vec<FileDiagnostic>), ProjectError> {
    let source = read::read_source(path)
        .map_err(|diagnostic| ProjectError::from_source(path, vec![diagnostic]))?;
    parse_compilation_unit_source(path, &source, source_id)
}

/// Returns the authoritative source bytes and AST, preserving source failures.
pub(super) fn read_compilation_unit_file(
    path: &Path,
    source_id: u32,
) -> Result<(Vec<u8>, CompilationUnit, Vec<FileDiagnostic>), ProjectError> {
    let source = read::read_source(path)
        .map_err(|diagnostic| ProjectError::from_source(path, vec![diagnostic]))?;
    let (unit, warnings) = parse_compilation_unit_source(path, &source, source_id)?;
    Ok((source, unit, warnings))
}

/// Parses a caller-owned snapshot and retains all diagnostics if parsing fails.
pub(super) fn parse_compilation_unit_source(
    path: &Path,
    source: &[u8],
    source_id: u32,
) -> Result<(CompilationUnit, Vec<FileDiagnostic>), ProjectError> {
    let source_text = std::str::from_utf8(source).map_err(|error| {
        ProjectError::from_source(
            path,
            vec![Diagnostic::error_without_source(
                fpas_diagnostics::codes::PROJECT_SOURCE_INVALID_UTF8,
                format!("Source file is not valid UTF-8: {error}"),
                Some("Save the source file as UTF-8 and retry.".to_owned()),
            )],
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

    if diagnostics.iter().any(Diagnostic::is_error) {
        return Err(ProjectError::from_source(path, diagnostics));
    }
    let warnings = diagnostics
        .into_iter()
        .map(|diagnostic| FileDiagnostic::new(diagnostic, Some(path.to_path_buf())))
        .collect();

    Ok((unit, warnings))
}

/// Warns that a `program` source was skipped; `rule` explains which sources are kept.
pub(super) fn program_source_skipped(
    path: &Path,
    program_name: &str,
    rule: &str,
) -> FileDiagnostic {
    FileDiagnostic::new(
        Diagnostic::warning_without_source(
            PROJECT_PROGRAM_SOURCE_SKIPPED,
            format!("Source file declares `program {program_name}` and was skipped."),
            Some(rule.to_owned()),
        ),
        Some(path.to_path_buf()),
    )
}

/// Rejects a second source file that declares an already-declared unit name.
pub(super) fn duplicate_unit_error(unit_name: &str, first: &Path, second: &Path) -> ProjectError {
    ProjectError::new(
        PROJECT_DUPLICATE_UNIT,
        format!(
            "Duplicate unit name `{unit_name}` found in `{}` and `{}`.",
            first.to_string_lossy(),
            second.to_string_lossy()
        ),
    )
    .with_help("Use a unique `unit` namespace per source file.")
}

pub(super) fn qualified_id_to_string(id: &QualifiedId) -> String {
    id.parts.join(".")
}

/// `docs/pascal/program-structure/units.md`: `Std.*` is reserved for implementation-defined standard units.
pub(super) fn validate_user_unit_name(path: &Path, id: &QualifiedId) -> Result<(), ProjectError> {
    if id
        .parts
        .first()
        .is_some_and(|head| head.eq_ignore_ascii_case("std"))
    {
        return Err(ProjectError::new(
            PROJECT_UNIT_NAMESPACE_INVALID,
            format!(
                "Source file `{}` declares `unit {}`.",
                path.to_string_lossy(),
                qualified_id_to_string(id)
            ),
        )
        .with_help(format!(
            "The root segment `Std` is reserved for standard library units. Rename the unit to a non-`Std` namespace such as `App.{}`.",
            id.parts.get(1).map_or("Core", String::as_str)
        )));
    }

    Ok(())
}
