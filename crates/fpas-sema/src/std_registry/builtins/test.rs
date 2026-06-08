//! Semantic checking for polymorphic `Std.Test` builtins.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use crate::check::Checker;
use crate::types::{ProcedureTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

/// Type-checks a `Std.Test` [`SymbolKind::BuiltinStd`] call when `name` matches.
pub(super) fn check_test_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    if name != s::STD_TEST_ASSERT_EQUALS {
        return None;
    }

    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 2 arguments, got {}",
                s::STD_TEST_ASSERT_EQUALS,
                args.len()
            ),
            "Example: AssertEquals(Expected, Actual).",
            span,
        );
        return Some(Ty::Unit);
    }

    let expected_ty = c.check_expr(&args[0]);
    let actual_ty = c.check_expr(&args[1]);
    if expected_ty != actual_ty {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`AssertEquals` expected and actual must have the same type".to_string(),
            "Pass two operands of the same type, for example two integers or two strings.",
            span,
        );
        return Some(Ty::Unit);
    }

    match expected_ty {
        Ty::Integer | Ty::Boolean | Ty::String | Ty::Real => Some(Ty::Unit),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                "`AssertEquals` supports integer, boolean, string, and real operands".to_string(),
                "Use AssertTrue or AssertFalse for boolean conditions without a separate expected value.",
                span,
            );
            Some(Ty::Unit)
        }
    }
}

/// Registers the polymorphic `AssertEquals` builtin placeholder.
pub(crate) fn register_assert_equals_builtin(checker: &mut Checker) {
    super::super::define_builtin_std(
        checker,
        s::STD_TEST_ASSERT_EQUALS,
        Ty::Procedure(ProcedureTy {
            type_params: Vec::new(),
            params: Vec::new(),
            variadic: false,
        }),
    );
}
