//! Canonical source formatter for Functional Pascal (AST pretty-printer).
//!
//! Normative style: [`docs/future/formater/style.md`](../../../docs/future/formater/style.md).
//! Language reference: [`docs/pascal/`](../../../docs/pascal/).

mod emit;
mod style;

use fpas_parser::{CompilationUnit, Program, Unit};

/// Formats a parsed compilation unit (`program` or `unit` file).
///
/// **Documentation:** `docs/future/formater/style.md`
#[must_use]
pub fn format_compilation_unit(unit: &CompilationUnit) -> String {
    match unit {
        CompilationUnit::Program(program) => format_program(program),
        CompilationUnit::Unit(unit) => format_unit(unit),
    }
}

/// Formats a `program` declaration and its body.
///
/// **Documentation:** `docs/future/formater/style.md`
#[must_use]
pub fn format_program(_program: &Program) -> String {
    todo!("Phase 5: emit program per docs/future/formater/style.md")
}

/// Formats a `unit` declaration and its declarations.
///
/// **Documentation:** `docs/future/formater/style.md`
#[must_use]
pub fn format_unit(_unit: &Unit) -> String {
    todo!("Phase 5: emit unit per docs/future/formater/style.md")
}
