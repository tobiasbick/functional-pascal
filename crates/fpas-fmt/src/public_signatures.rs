//! Source-derived public declarations for the read-only editor view.

use fpas_parser::{CompilationUnit, Unit, parse_compilation_unit};

use crate::{FormatError, comments::CommentMap, emit};

/// Formats a unit's public signatures without routine bodies or private members.
///
/// The result is editor documentation, not a compilable unit. Binding initializers
/// and comments are omitted; public field defaults and declared contracts remain.
/// **Documentation:** `docs/pascal/tools/editor-integration.md#public-signatures`
///
/// # Errors
/// Returns a source mismatch or span error when `unit` does not match `source`.
pub fn format_public_signatures(source: &str, unit: &Unit) -> Result<String, FormatError> {
    let (parsed, _) = parse_compilation_unit(source);
    if !matches!(&parsed, CompilationUnit::Unit(value) if value == unit) {
        return Err(FormatError::SourceMismatch);
    }
    let comments = CommentMap::build(source, &parsed)?;
    Ok(emit::format_public_signatures(unit, &comments))
}
