mod array;
mod callback_context;
mod callbacks;
mod channel_task;
mod dict;
mod math;
mod result_option;
mod str_ops;
mod test;

pub(super) use test::register_assert_equals_builtin;

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{INTERNAL_COMPILER_INVARIANT_FAILURE, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;

pub fn check_builtin_std_call(c: &mut Checker, name: &str, args: &[Expr], span: Span) -> Ty {
    check_builtin_std_call_refs(c, name, &args.iter().collect::<Vec<_>>(), span)
}

/// Check an intrinsic using the original argument nodes, preserving explicit source arguments.
pub fn check_builtin_std_call_refs(c: &mut Checker, name: &str, args: &[&Expr], span: Span) -> Ty {
    let pure = fpas_std::intrinsic_std_function_is_pure(name);
    if !pure {
        c.reject_impure_operation(&format!("call ordinary routine `{name}`"), span);
    }
    let previous = callback_context::prepare(c, name, args);
    let result = check_builtin_operation(c, name, args, span);
    callback_context::restore(c, previous);
    if pure {
        for (index, arg) in args.iter().enumerate() {
            if let Some(ty) = c.expr_types.get(&Checker::expr_lookup_key(arg))
                && !c.is_pure_data(ty)
            {
                c.error_with_code(fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!("Argument {} of pure intrinsic `{name}` is not resource-free data or a pure callable", index + 1),
                    "Use resource-free data and explicitly pure function callbacks; use action APIs for effects.", arg.span());
            }
        }
        if !c.is_pure_data(&result) {
            c.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                format!("Pure intrinsic `{name}` cannot return `{result}`"),
                "Use resource-free data or pure callable results.",
                span,
            );
        }
    }
    result
}

fn check_builtin_operation(c: &mut Checker, name: &str, args: &[&Expr], span: Span) -> Ty {
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
    if let Some(ty) = str_ops::check_str_builtin_std_call(c, name, args, span) {
        return ty;
    }
    if let Some(ty) = test::check_test_builtin_std_call(c, name, args, span) {
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

fn check_argument_count(
    c: &mut Checker,
    name: &str,
    expected: usize,
    args: &[&Expr],
    example: &str,
    span: Span,
) -> bool {
    if args.len() == expected {
        return true;
    }
    c.error_with_code(
        SEMA_WRONG_ARGUMENT_COUNT,
        format!("`{name}` expects {expected} arguments, got {}", args.len()),
        example,
        span,
    );
    false
}

pub(super) fn array_elem_ty(ty: &Ty) -> Option<Ty> {
    if let Ty::Array(inner) = ty {
        Some(*inner.clone())
    } else {
        None
    }
}
