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
mod interface;
mod scope;
mod std_registry;
mod std_units;
mod types;

pub use check::AnalysisMetadata;
pub use check::BoundMethodInfo;
pub use check::BoundMethodMap;
pub use check::CaptureBinding;
pub use check::ClosureInfo;
pub use check::ClosureInfoMap;
pub use check::EventAssignedInfo;
pub use check::EventAssignedMap;
pub use check::EventRaiseInfo;
pub use check::EventRaiseMap;
pub use check::EventWriteInfo;
pub use check::EventWriteMap;
pub use check::ExprTypeMap;
pub use check::IntrinsicCallMap;
pub use check::MethodCallMap;
pub use check::MethodCallTarget;
pub use check::NamedTypeMap;
pub use check::NestedRoutineCaptureInfo;
pub use check::NestedRoutineCaptureMap;
pub use check::PropertyReadInfo;
pub use check::PropertyReadMap;
pub use check::PropertyWriteInfo;
pub use check::PropertyWriteMap;
pub use check::RecordDefaultsMap;
pub use check::ScalarCaseBindingMap;
pub use error::SemaError;
pub use interface::{
    InterfaceConversionError, UnitAnalysis, analyze_program_with_interface_support,
    analyze_program_with_interfaces, analyze_unit, analyze_unit_with_interface_support,
    interface_type_to_ty, ty_to_interface_type,
};
pub use std_registry::{
    IntrinsicStdSymbol, IntrinsicStdSymbolKind, intrinsic_std_symbols, intrinsic_std_units,
};
pub use types::{EnumTy, FunctionTy, ParamTy, ProcedureTy, RecordTy, Ty};

use fpas_parser::Program;

/// Run semantic analysis on a parsed program.
/// Returns a list of diagnostics (may be empty on success).
pub fn analyze(program: &Program) -> Vec<SemaError> {
    analyze_with_types(program).errors
}

/// Like [`analyze`], but also returns the inferred type of every expression (by source key)
/// and the map of record type defaults used by the compiler for default field expansion.
pub fn analyze_with_types(program: &Program) -> AnalysisMetadata {
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

/// Stable key for call-statement method resolution (address of the call's
/// [`Designator`](fpas_parser::Designator) in the AST).
///
/// **Documentation:** `docs/pascal/language/functions/README.md` (record method calls; from the repository root).
#[must_use]
pub fn designator_lookup_key(designator: &fpas_parser::Designator) -> usize {
    std::ptr::from_ref(designator) as usize
}

#[cfg(test)]
mod tests;
