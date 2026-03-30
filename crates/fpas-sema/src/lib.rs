#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "semantic analysis tests use expect to keep diagnostic assertions short"
    )
)]

mod check;
mod error;
mod scope;
mod std_registry;
mod std_units;
mod types;

pub use check::ExprTypeMap;
pub use check::InterfaceDispatchMap;
pub use check::MethodCallMap;
pub use check::RecordDefaultsMap;
pub use error::SemaError;
pub use types::Ty;

use fpas_parser::Program;

/// Run semantic analysis on a parsed program.
/// Returns a list of diagnostics (may be empty on success).
pub fn analyze(program: &Program) -> Vec<SemaError> {
    analyze_with_types(program).0
}

/// Like [`analyze`], but also returns the inferred type of every expression (by source key)
/// and the map of record type defaults used by the compiler for default field expansion.
pub fn analyze_with_types(
    program: &Program,
) -> (
    Vec<SemaError>,
    ExprTypeMap,
    MethodCallMap,
    InterfaceDispatchMap,
    RecordDefaultsMap,
) {
    let mut checker = check::Checker::new();
    checker.check_program(program);
    checker.finish()
}

/// Stable key for looking up [`ExprTypeMap`] entries (address of the `Expr` in the AST).
#[must_use]
pub fn expr_lookup_key(expr: &fpas_parser::Expr) -> usize {
    check::Checker::expr_lookup_key(expr)
}

#[cfg(test)]
mod tests;
