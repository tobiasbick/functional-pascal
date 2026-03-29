mod array;
mod channel_task;
mod dict;
mod math;
mod result_option;

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::INTERNAL_COMPILER_INVARIANT_FAILURE;
use fpas_lexer::Span;
use fpas_parser::{DesignatorPart, Expr};

pub fn check_builtin_std_call(c: &mut Checker, name: &str, args: &[Expr], span: Span) -> Ty {
    if let Some(ty) = array::check_array_builtin_std_call(c, name, args, span) {
        return ty;
    }
    if let Some(ty) = channel_task::check_channel_task_builtin_std_call(c, name, args, span) {
        return ty;
    }
    if let Some(ty) = dict::check_dict_builtin_std_call(c, name, args, span) {
        return ty;
    }
    if let Some(ty) = math::check_math_builtin_std_call(c, name, args, span) {
        return ty;
    }
    if let Some(ty) = result_option::check_result_option_builtin_std_call(c, name, args, span) {
        return ty;
    }

    c.error_with_code(
        INTERNAL_COMPILER_INVARIANT_FAILURE,
        format!("Internal: unknown BuiltinStd `{name}`"),
        "Report this as a compiler bug.",
        span,
    );
    Ty::Error
}

pub(super) fn simple_var_name(expr: &Expr) -> Option<String> {
    let Expr::Designator(d) = expr else {
        return None;
    };
    if d.parts.len() != 1 {
        return None;
    }
    match &d.parts[0] {
        DesignatorPart::Ident(name, _) => Some(name.clone()),
        _ => None,
    }
}

pub(super) fn mutable_array_elem_ty(c: &Checker, name: &str) -> Option<Ty> {
    let sym = c.scopes.lookup(name)?;
    if !sym.mutable {
        return None;
    }
    match &sym.ty {
        Ty::Array(elem) => Some(*elem.clone()),
        _ => None,
    }
}

pub(super) fn array_elem_ty(ty: &Ty) -> Option<Ty> {
    if let Ty::Array(inner) = ty {
        Some(*inner.clone())
    } else {
        None
    }
}
