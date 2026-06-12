//! Canonical source formatter for Functional Pascal (AST pretty-printer).
//!
//! Normative style: [`docs/future/formater/style.md`](../../../docs/future/formater/style.md).
//! Language reference: [`docs/pascal/`](../../../docs/pascal/).

mod comments;
mod emit;
mod span;
mod style;

use comments::CommentMap;
use emit::{format_program as emit_program, format_unit as emit_unit};
use fpas_parser::{CompilationUnit, Program, Unit};

/// Formats a parsed compilation unit without access to original source (comments stripped).
///
/// Prefer [`format_source`] when the original text is available.
///
/// **Documentation:** `docs/future/formater/style.md`
#[must_use]
pub fn format_compilation_unit(unit: &CompilationUnit) -> String {
    format_with_comments(unit, &CommentMap::default())
}

/// Formats `unit` using `source` to preserve leading doc and declaration comments.
///
/// **Documentation:** `docs/future/formater/style.md#comments`
#[must_use]
pub fn format_source(source: &str, unit: &CompilationUnit) -> String {
    format_with_comments(unit, &CommentMap::build(source, unit))
}

/// Formats a `program` declaration and its body.
///
/// **Documentation:** `docs/future/formater/style.md`
#[must_use]
pub fn format_program(program: &Program) -> String {
    emit_program(program, &CommentMap::default())
}

/// Formats a `unit` declaration and its declarations.
///
/// **Documentation:** `docs/future/formater/style.md`
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
