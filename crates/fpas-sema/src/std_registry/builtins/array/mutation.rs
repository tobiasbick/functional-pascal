//! Caller-mutating arrays use the shared `var` storage and argument checks.
//!
//! **Documentation:** `docs/pascal/language/types/array/mutating.md`

use crate::check::Checker;
use crate::types::{ParamTy, Ty};
use fpas_diagnostics::codes::{SEMA_INVALID_VAR_ARGUMENT, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::Expr;

use super::check_argument_count;

/// Checks append with a writable receiver and a read-only value argument.
pub(super) fn check_push(
    c: &mut Checker,
    name: &str,
    args: &[&Expr],
    implicit_receiver: bool,
    span: Span,
) -> Ty {
    if !check_argument_count(c, name, 2, args, "Example: Items.Push(Value).", span) {
        return Ty::Error;
    }
    let element = check_receiver(c, name, args[0], implicit_receiver, span);
    let param = ParamTy::value("Value", element.clone());
    let (value, _) = c.check_argument_for_param(name, &param, args[1], false);
    c.check_type_compat(&element, &value, "pushed value", args[1].span());
    Ty::Unit
}

/// Checks removal and returns the writable array receiver's element type.
pub(super) fn check_pop(
    c: &mut Checker,
    name: &str,
    args: &[&Expr],
    implicit_receiver: bool,
    span: Span,
) -> Ty {
    if !check_argument_count(c, name, 1, args, "Example: Items.Pop().", span) {
        return Ty::Error;
    }
    check_receiver(c, name, args[0], implicit_receiver, span)
}

fn check_receiver(c: &mut Checker, name: &str, arg: &Expr, implicit: bool, span: Span) -> Ty {
    let (ty, root) = if implicit {
        let ty = c.check_expr(arg);
        let Expr::Designator(designator) = arg else {
            c.error_with_code(
                SEMA_INVALID_VAR_ARGUMENT,
                format!("`{name}` requires a writable array receiver"),
                "Use a `var` array variable, record field, array element, or forwarded `var` parameter, for example `Items.Push(Value)` or `Items.Pop()`.",
                arg.span(),
            );
            return Ty::Error;
        };
        c.check_var_storage(designator, ty)
    } else {
        let param = ParamTy {
            name: "A".to_string(),
            ty: Ty::Error,
            mode: crate::std_registry::intrinsic_std_receiver_mode(name),
        };
        c.check_argument_for_param(name, &param, arg, false)
    };
    let Some(root) = root else {
        return Ty::Error;
    };
    c.reject_var_argument_aliases(&[root], span);
    match ty {
        Ty::Array(element) => *element,
        Ty::Error => Ty::Error,
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{name}` requires an array, got `{other}`"),
                "Declare the receiver as `var Items: array of T := …;`.",
                arg.span(),
            );
            Ty::Error
        }
    }
}
