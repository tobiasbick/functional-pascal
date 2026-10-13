//! Value-producing array append and pop checks.
//! Documentation: `docs/pascal/language/types/array/mutating.md`.

use super::{array_elem_ty, check_argument_count};
use crate::check::Checker;
use crate::types::{ParamTy, Ty};
use fpas_lexer::Span;
use fpas_parser::Expr;

/// Checks a value append while retaining the receiver's element type.
pub(super) fn check_push(c: &mut Checker, name: &str, args: &[&Expr], span: Span) -> Ty {
    if !check_argument_count(c, name, 2, args, "Use Items.Push(Value).", span) {
        return Ty::Error;
    }
    let receiver = c.check_expr(args[0]);
    let Some(element) = array_elem_ty(&receiver) else {
        return Ty::Error;
    };
    let (value, _) = c.check_argument_for_param(
        name,
        &ParamTy::value("Value", element.clone()),
        args[1],
        false,
    );
    c.check_type_compat(&element, &value, "pushed value", args[1].span());
    receiver
}

/// Checks value pop and specializes its named result record.
pub(super) fn check_pop(c: &mut Checker, name: &str, args: &[&Expr], span: Span) -> Ty {
    if !check_argument_count(c, name, 1, args, "Use Items.Pop().", span) {
        return Ty::Error;
    }
    array_elem_ty(&c.check_expr(args[0])).map_or(Ty::Error, crate::types::array_pop_result)
}
