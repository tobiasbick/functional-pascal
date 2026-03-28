use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::{array_elem_ty, check_argument_count};

/// `Std.Array.Map(Arr, F)` -> `array of U` where `F: function(V: T): U`.
pub(super) fn check_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
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
    let Some(_elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_MAP),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };

    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => Ty::Array(return_type.clone()),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{}` second argument must be a function", s::STD_ARRAY_MAP),
                "Pass a function or anonymous function: function(X: T): U begin ... end.",
                span,
            );
            Ty::Error
        }
    }
}

/// `Std.Array.Filter(Arr, F)` -> `array of T` where `F: function(V: T): boolean`.
pub(super) fn check_filter(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
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
    if array_elem_ty(&arr_ty).is_none() {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_FILTER),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    }
    if !matches!(func_ty, Ty::Function(_)) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` second argument must be a function",
                s::STD_ARRAY_FILTER
            ),
            "Pass a function(V: T): boolean.",
            span,
        );
        return Ty::Error;
    }

    arr_ty
}

/// `Std.Array.Reduce(Arr, Init, F)` -> `U` where `F: function(Acc: U; V: T): U`.
pub(super) fn check_reduce(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
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
    if array_elem_ty(&arr_ty).is_none() {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_REDUCE),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    }

    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => *return_type.clone(),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{}` third argument must be a function",
                    s::STD_ARRAY_REDUCE
                ),
                "Pass a function(Acc: U; V: T): U.",
                span,
            );
            let _ = init_ty;
            Ty::Error
        }
    }
}
