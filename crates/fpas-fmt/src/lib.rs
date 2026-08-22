//! Canonical source formatter for Functional Pascal (AST pretty-printer).
//!
//! Normative style: [`docs/pascal/tools/fmt-style.md`](../../../docs/pascal/tools/fmt-style.md).
//! Language reference: [`docs/pascal/`](../../../docs/pascal/).

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "formatter tests use unwrap/expect/panic to keep fixture assertions focused"
    )
)]

mod comments;
mod emit;
mod error;
mod span;
mod style;

use comments::CommentMap;
use emit::{format_program as emit_program, format_unit as emit_unit};
use fpas_parser::{CompilationUnit, Program, Unit};

pub use error::FormatError;

/// Formats a parsed compilation unit without access to original source.
///
/// Comments cannot be preserved without source text. Prefer [`format_source`] for `fpas fmt`
/// and any workflow where comments must be kept.
///
/// **Documentation:** `docs/pascal/tools/fmt-style.md`
#[must_use]
pub fn format_compilation_unit(unit: &CompilationUnit) -> String {
    format_with_comments(unit, &CommentMap::default())
}

/// Formats `unit` using `source` to preserve every `//` comment.
///
/// **Documentation:** `docs/pascal/tools/fmt-style.md#comments`
///
/// # Errors
///
/// Returns [`FormatError::SourceMismatch`] when `unit` was parsed from different text, or
/// [`FormatError::InvalidSourceSpan`] when a supplied span is outside the source or splits a
/// UTF-8 code point.
pub fn format_source(source: &str, unit: &CompilationUnit) -> Result<String, FormatError> {
    let comments = CommentMap::build(source, unit)?;
    let (parsed, _) = fpas_parser::parse_compilation_unit(source);
    if &parsed != unit {
        return Err(FormatError::SourceMismatch);
    }
    Ok(format_with_comments(unit, &comments))
}

/// Formats a `program` declaration and its body.
///
/// **Documentation:** `docs/pascal/tools/fmt-style.md`
#[must_use]
pub fn format_program(program: &Program) -> String {
    emit_program(program, &CommentMap::default())
}

/// Formats a `unit` declaration and its declarations.
///
/// **Documentation:** `docs/pascal/tools/fmt-style.md`
#[must_use]
pub fn format_unit(unit: &Unit) -> String {
    emit_unit(unit, &CommentMap::default())
}

fn format_with_comments(unit: &CompilationUnit, comments: &CommentMap) -> String {
    match unit {
        CompilationUnit::Program(program) => emit_program(program, comments),
        CompilationUnit::Unit(unit) => emit_unit(unit, comments),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_parser::parse_compilation_unit;

    #[test]
    fn format_source_preserves_all_comments() {
        let source = "// Unit doc.\nunit Demo;\n\n// field doc\nmutable var Count: integer := 0;\n";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");

        let without_source = format_compilation_unit(&unit);
        assert!(
            !without_source.contains("Unit doc"),
            "AST-only formatting cannot recover comments without source"
        );

        let with_source = format_source(source, &unit).expect("matching source and AST");
        assert!(with_source.contains("// Unit doc."));
        assert!(with_source.contains("// field doc"));
    }

    #[test]
    fn format_source_preserves_end_of_line_comments() {
        let source = "program T; begin\n  WriteLn('ok') // trail\nend.";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let formatted = format_source(source, &unit).expect("matching source and AST");
        assert!(formatted.contains("// trail"));
    }

    #[test]
    fn format_source_preserves_documentation_attachment_and_detachment() {
        for (source, expected) in [
            (
                "// attached\nprogram T; begin end.",
                "// attached\nprogram T;",
            ),
            (
                "// detached\n\nprogram T; begin end.",
                "// detached\n\nprogram T;",
            ),
        ] {
            let (unit, errors) = parse_compilation_unit(source);
            assert!(errors.is_empty(), "{errors:?}");
            let formatted = format_source(source, &unit).expect("matching source and AST");
            assert!(formatted.starts_with(expected), "{formatted:?}");
        }
    }
}
