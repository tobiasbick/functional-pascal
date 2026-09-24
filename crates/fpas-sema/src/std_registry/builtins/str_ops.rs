//! Type checks for `Std.Str` operations that invoke FPAS callbacks.
//!
//! **Documentation:** `docs/pascal/std/text/str/higher-order.md` (from the repository root).

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::callbacks::{
    BinaryFunctionCallbackSpec, expect_binary_function_callback, expect_unary_function_callback,
};
use super::check_argument_count;

pub(super) fn check_str_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    let ty = match name {
        s::STD_STR_MAP => check_map_or_filter(c, name, args, span, Ty::String),
        s::STD_STR_FILTER => check_map_or_filter(c, name, args, span, Ty::Boolean),
        s::STD_STR_REDUCE => check_reduce(c, args, span),
        _ => return None,
    };
    Some(ty)
}

fn check_map_or_filter(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
    callback_result: Ty,
) -> Ty {
    if !check_argument_count(
        c,
        name,
        2,
        args,
        "Pass a string and a function(C: string) with the documented result type.",
        span,
    ) {
        return Ty::Error;
    }
    let input = c.check_expr(&args[0]);
    let callback = c.check_expr(&args[1]);
    if input == Ty::Error || callback == Ty::Error {
        return Ty::Error;
    }
    if !Ty::String.compatible_with(&input) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{name}` first argument must be a string"),
            "Pass a string as the first argument.",
            span,
        );
        return Ty::Error;
    }
    if expect_unary_function_callback(
        c,
        name,
        &callback,
        &Ty::String,
        Some(&callback_result),
        span,
        "Pass a function(C: string): string to Map or a function(C: string): boolean to Filter.",
    )
    .is_some()
    {
        Ty::String
    } else {
        Ty::Error
    }
}

fn check_reduce(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_STR_REDUCE,
        3,
        args,
        "Pass Reduce(S, Init, function(Acc: U; C: string): U).",
        span,
    ) {
        return Ty::Error;
    }
    let input = c.check_expr(&args[0]);
    let initial = c.check_expr(&args[1]);
    let callback = c.check_expr(&args[2]);
    if input == Ty::Error || initial == Ty::Error || callback == Ty::Error {
        return Ty::Error;
    }
    if !Ty::String.compatible_with(&input) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be a string", s::STD_STR_REDUCE),
            "Pass a string as the first argument.",
            span,
        );
        return Ty::Error;
    }
    expect_binary_function_callback(
        c,
        s::STD_STR_REDUCE,
        &callback,
        BinaryFunctionCallbackSpec {
            first_param_ty: &initial,
            second_param_ty: &Ty::String,
            return_ty: Some(&initial),
            hint: "Pass a function(Acc: U; C: string): U.",
        },
        span,
    )
    .unwrap_or(Ty::Error)
}
