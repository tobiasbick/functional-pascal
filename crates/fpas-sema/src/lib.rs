#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "semantic analysis tests use expect to keep diagnostic assertions short"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::panic,
        reason = "semantic analysis tests use explicit panic for structural mismatches"
    )
)]

mod check;
mod error;
mod scope;
mod std_registry;
mod std_units;
mod types;

pub use check::BoundMethodInfo;
pub use check::BoundMethodMap;
pub use check::CaptureBinding;
pub use check::ClosureInfo;
pub use check::ClosureInfoMap;
pub use check::ExprTypeMap;
pub use check::MethodCallMap;
pub use check::MethodCallTarget;
pub use check::NestedRoutineCaptureInfo;
pub use check::NestedRoutineCaptureMap;
pub use check::RecordDefaultsMap;
pub use check::ScalarCaseBindingMap;
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
    RecordDefaultsMap,
    ScalarCaseBindingMap,
    ClosureInfoMap,
    NestedRoutineCaptureMap,
    BoundMethodMap,
) {
    let mut checker = check::Checker::new();
    checker.check_program(program);
    checker.finish()
}

/// Stable key for looking up [`ExprTypeMap`] entries (address of the `Expr` in the AST).
///
/// Uses the memory address of the `Expr` reference. This is sound because the AST is immutable
/// for the whole compile pipeline; keys must match between sema and codegen for the same tree.
#[must_use]
pub fn expr_lookup_key(expr: &fpas_parser::Expr) -> usize {
    check::Checker::expr_lookup_key(expr)
}

/// Stable key for looking up [`MethodCallMap`] entries for a postfix method operation.
///
/// Uses the memory address of the [`fpas_parser::PostfixOperation`] in the AST. Same soundness
/// rationale as [`expr_lookup_key`]: the AST is immutable for the compile pipeline, so keys must
/// match between sema and codegen for the same tree.
///
/// **Documentation:** `docs/pascal/language/functions/README.md`
#[must_use]
pub fn postfix_operation_lookup_key(op: &fpas_parser::PostfixOperation) -> usize {
    check::Checker::postfix_operation_lookup_key(op)
}

/// Stable key for call-statement method resolution (address of the call's [`Designator`] in the AST).
///
/// **Documentation:** `docs/pascal/language/functions/README.md` (record method calls; from the repository root).
#[must_use]
pub fn designator_lookup_key(designator: &fpas_parser::Designator) -> usize {
    std::ptr::from_ref(designator) as usize
}

#[cfg(test)]
mod tests;
