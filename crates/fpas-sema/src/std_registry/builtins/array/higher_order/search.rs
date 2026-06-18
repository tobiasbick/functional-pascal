//! Higher-order `Std.Array` semantic checks: `Find`, `FindIndex`, `Any`, `All`, `ForEach`.
//!
//! **Documentation:** `docs/pascal/std/collections/array.md` (from the repository root).

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::{array_elem_ty, check_argument_count};
use super::callbacks::{expect_unary_function_callback, expect_unary_procedure_callback};

/// `Std.Array.Find(Arr, Pred)` → `option of T` where `Pred: function(V: T): boolean`.
pub(crate) fn check_find(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FIND,
        2,
        args,
        "Example: Std.Array.Find(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_FIND),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_FIND,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Option(Box::new(elem_ty))
}

/// `Std.Array.FindIndex(Arr, Pred)` → `integer`.
pub(crate) fn check_find_index(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FIND_INDEX,
        2,
        args,
        "Example: Std.Array.FindIndex(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` first argument must be an array",
                s::STD_ARRAY_FIND_INDEX
            ),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_FIND_INDEX,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Integer
}

/// `Std.Array.Any(Arr, Pred)` → `boolean`.
pub(crate) fn check_any(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_ANY,
        2,
        args,
        "Example: Std.Array.Any(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_ANY),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_ANY,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Boolean
}

/// `Std.Array.All(Arr, Pred)` → `boolean`.
pub(crate) fn check_all(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_ALL,
        2,
        args,
        "Example: Std.Array.All(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_ALL),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_ALL,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Boolean
}

/// `Std.Array.ForEach(Arr, F)` → `unit`.
pub(crate) fn check_for_each(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FOR_EACH,
        2,
        args,
        "Example: Std.Array.ForEach(Arr, procedure(X: integer) begin ... end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` first argument must be an array",
                s::STD_ARRAY_FOR_EACH
            ),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_procedure_callback(
        c,
        s::STD_ARRAY_FOR_EACH,
        &func_ty,
        &elem_ty,
        span,
        "Pass a procedure(V: T).",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Unit
}
