use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::{check_argument_count, mutable_array_elem_ty, simple_var_name};

pub(super) fn check_push(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_PUSH,
        2,
        args,
        "Example: Std.Array.Push(Arr, Value). The array must be a `mutable var`.",
        span,
    ) {
        return Ty::Error;
    }

    let Some(var_name) = simple_var_name(&args[0]) else {
        c.error_with_code(
            SEMA_IMMUTABLE_ASSIGNMENT,
            format!(
                "`{}` first argument must be a simple mutable array variable",
                s::STD_ARRAY_PUSH
            ),
            "Use `mutable var N: array of T := [...]` then `Std.Array.Push(N, x)`.",
            span,
        );
        c.check_expr(&args[1]);
        return Ty::Error;
    };
    let Some(elem_ty) = mutable_array_elem_ty(c, &var_name) else {
        c.error_with_code(
            SEMA_IMMUTABLE_ASSIGNMENT,
            format!("`{var_name}` must be a `mutable var` of array type"),
            "Declare with `mutable var Name: array of T := ...`.",
            span,
        );
        c.check_expr(&args[1]);
        return Ty::Error;
    };

    let value_ty = c.check_expr(&args[1]);
    c.check_type_compat(&elem_ty, &value_ty, "pushed value", span);
    Ty::Unit
}

pub(super) fn check_pop(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_POP,
        1,
        args,
        "Example: Std.Array.Pop(Arr).",
        span,
    ) {
        return Ty::Error;
    }

    let Some(var_name) = simple_var_name(&args[0]) else {
        c.error_with_code(
            SEMA_IMMUTABLE_ASSIGNMENT,
            format!(
                "`{}` argument must be a simple mutable array variable",
                s::STD_ARRAY_POP
            ),
            "Use `mutable var N: array of T := [...]` then `Std.Array.Pop(N)`.",
            span,
        );
        return Ty::Error;
    };
    let Some(elem_ty) = mutable_array_elem_ty(c, &var_name) else {
        c.error_with_code(
            SEMA_IMMUTABLE_ASSIGNMENT,
            format!("`{var_name}` must be a `mutable var` of array type"),
            "Declare with `mutable var Name: array of T := ...`.",
            span,
        );
        return Ty::Error;
    };

    elem_ty
}
