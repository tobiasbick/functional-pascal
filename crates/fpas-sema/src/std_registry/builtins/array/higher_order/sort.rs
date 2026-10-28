//! Key-selector ordering checks; see `docs/pascal/language/types/array/basics.md`.

use crate::check::Checker;
use crate::types::Ty;
use fpas_builtins::std_symbols as s;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;

use super::super::super::callbacks::expect_unary_function_callback;
use super::super::{array_elem_ty, check_argument_count};

/// Check `Items.SortBy(F)` with unconstrained elements and Comparable keys.
pub(crate) fn check_sort_by(c: &mut Checker, args: &[&Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_SORT_BY,
        2,
        args,
        "Use Items.SortBy(F).",
        span,
    ) {
        return Ty::Error;
    }
    let array_ty = c.check_expr(args[0]);
    let callback_ty = c.check_expr(args[1]);
    let Some(element_ty) = array_elem_ty(&array_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "SortBy requires an array receiver",
            "Use Items.SortBy(F) with Items: array of T.",
            span,
        );
        return Ty::Error;
    };
    let Some(key_ty) = expect_unary_function_callback(
        c,
        s::STD_ARRAY_SORT_BY,
        &callback_ty,
        &element_ty,
        None,
        span,
        "Pass a function(X: T): K with K: Comparable.",
    ) else {
        return Ty::Error;
    };
    if key_ty.is_error() {
        return Ty::Error;
    }
    if !key_ty.is_comparable() {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("SortBy requires Comparable keys, got `{key_ty}`"),
            "Return an integer, real, Boolean, string, subrange, distinct scalar or payload-free enum key; require K: Comparable (or K: Numeric) for generic keys.",
            span,
        );
        return Ty::Error;
    }
    array_ty
}
