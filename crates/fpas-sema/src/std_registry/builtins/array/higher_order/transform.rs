//! Higher-order `Std.Array` semantic checks: `Map`, `Filter`, `Reduce`, `FlatMap`.
//!
//! **Documentation:** `docs/pascal/std/collections/array.md` (from the repository root).

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::{array_elem_ty, check_argument_count};
use super::callbacks::{
    BinaryFunctionCallbackSpec, expect_binary_function_callback, expect_unary_function_callback,
};

/// `Std.Array.Map(Arr, F)` → `array of U` where `F: function(V: T): U`.
pub(crate) fn check_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_MAP,
        2,
        args,
        "Example: Std.Array.Map(Arr, function(X: integer): integer begin return X * 2 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_MAP),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };

    match expect_unary_function_callback(
        c,
        s::STD_ARRAY_MAP,
        &func_ty,
        &elem_ty,
        None,
        span,
        "Pass a function(X: T): U.",
    ) {
        Some(return_type) => Ty::Array(Box::new(return_type)),
        _ => Ty::Error,
    }
}

/// `Std.Array.Filter(Arr, F)` → `array of T` where `F: function(V: T): boolean`.
pub(crate) fn check_filter(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FILTER,
        2,
        args,
        "Example: Std.Array.Filter(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_FILTER),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_FILTER,
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

    arr_ty
}

/// `Std.Array.Reduce(Arr, Init, F)` → `U` where `F: function(Acc: U; V: T): U`.
pub(crate) fn check_reduce(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_REDUCE,
        3,
        args,
        "Example: Std.Array.Reduce(Arr, 0, function(Acc: integer; V: integer): integer begin return Acc + V end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let init_ty = c.check_expr(&args[1]);
    let func_ty = c.check_expr(&args[2]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_REDUCE),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };

    match expect_binary_function_callback(
        c,
        s::STD_ARRAY_REDUCE,
        &func_ty,
        BinaryFunctionCallbackSpec {
            first_param_ty: &init_ty,
            second_param_ty: &elem_ty,
            return_ty: Some(&init_ty),
            hint: "Pass a function(Acc: U; V: T): U.",
        },
        span,
    ) {
        Some(return_ty) => return_ty,
        None => Ty::Error,
    }
}

/// `Std.Array.FlatMap(Arr, F)` → `array of U` where `F: function(V: T): array of U`.
pub(crate) fn check_flat_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FLAT_MAP,
        2,
        args,
        "Example: Std.Array.FlatMap(Arr, function(X: integer): array of integer begin ... end).",
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
                s::STD_ARRAY_FLAT_MAP
            ),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    match expect_unary_function_callback(
        c,
        s::STD_ARRAY_FLAT_MAP,
        &func_ty,
        &elem_ty,
        None,
        span,
        "Pass a function(V: T): array of U.",
    ) {
        Some(return_ty) if return_ty.is_error() => Ty::Error,
        Some(return_ty) if array_elem_ty(&return_ty).is_some() => return_ty,
        Some(_) => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{}` mapper must return an array", s::STD_ARRAY_FLAT_MAP),
                "Pass a function(V: T): array of U.",
                span,
            );
            Ty::Error
        }
        None => Ty::Error,
    }
}
