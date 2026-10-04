//! Array mutations use the ordinary explicit caller-storage contract.
//!
//! Documentation: `docs/pascal/std/collections/array/mutating.md`.

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::check_argument_count;

pub(super) fn check_push(c: &mut Checker, args: &[&Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_PUSH,
        2,
        args,
        "Example: Std.Arrays.Push(var Arr, Value).",
        span,
    ) {
        return Ty::Error;
    }
    let elem_ty = selected_array_element(c, args[0]);
    let value_ty = c.check_expr_with_expected(args[1], &elem_ty);
    c.check_type_compat(&elem_ty, &value_ty, "pushed value", span);
    Ty::Unit
}

pub(super) fn check_pop(c: &mut Checker, args: &[&Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_POP,
        1,
        args,
        "Example: Std.Arrays.Pop(var Arr).",
        span,
    ) {
        return Ty::Error;
    }
    selected_array_element(c, args[0])
}

fn selected_array_element(c: &mut Checker, argument: &Expr) -> Ty {
    match c.check_var_argument(argument) {
        Ty::Array(element) => *element,
        Ty::Error => Ty::Error,
        ty => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Array mutation requires array storage, got `{ty}`"),
                "Pass an array binding, stored field or element as `var Target`.",
                argument.span(),
            );
            Ty::Error
        }
    }
}
