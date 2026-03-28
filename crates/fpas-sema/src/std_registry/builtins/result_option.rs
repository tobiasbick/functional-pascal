//! Type-checking builtins for `Std.Result` and `Std.Option` calls.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

pub(super) fn check_result_option_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    let ty = match name {
        s::STD_RESULT_UNWRAP => check_one_arg(c, name, args, span, unwrap_result_ok),
        s::STD_RESULT_UNWRAP_OR => {
            check_two_args(c, name, args, span, |ty, _| unwrap_result_ok(ty))
        }
        s::STD_RESULT_IS_OK | s::STD_RESULT_IS_ERR => {
            check_one_arg(c, name, args, span, |_| Ty::Boolean)
        }
        s::STD_OPTION_UNWRAP => check_one_arg(c, name, args, span, unwrap_option),
        s::STD_OPTION_UNWRAP_OR => check_two_args(c, name, args, span, |ty, _| unwrap_option(ty)),
        s::STD_OPTION_IS_SOME | s::STD_OPTION_IS_NONE => {
            check_one_arg(c, name, args, span, |_| Ty::Boolean)
        }
        _ => return None,
    };
    Some(ty)
}

fn unwrap_result_ok(ty: Ty) -> Ty {
    match ty {
        Ty::Result(ok, _) => *ok,
        _ => Ty::Error,
    }
}

fn unwrap_option(ty: Ty) -> Ty {
    match ty {
        Ty::Option(inner) => *inner,
        _ => Ty::Error,
    }
}

fn check_one_arg(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
    derive: impl FnOnce(Ty) -> Ty,
) -> Ty {
    if args.len() != 1 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{name}` expects 1 argument, got {}", args.len()),
            format!("Call `{name}` with exactly 1 argument."),
            span,
        );
        return Ty::Error;
    }
    let ty = c.check_expr(&args[0]);
    derive(ty)
}

fn check_two_args(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
    derive: impl FnOnce(Ty, Ty) -> Ty,
) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{name}` expects 2 arguments, got {}", args.len()),
            format!("Call `{name}` with exactly 2 arguments."),
            span,
        );
        return Ty::Error;
    }
    let ty1 = c.check_expr(&args[0]);
    let ty2 = c.check_expr(&args[1]);
    derive(ty1, ty2)
}
