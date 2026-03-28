use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

pub(super) fn check_math_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    let ty = match name {
        s::STD_MATH_ABS => check_abs(c, args, span),
        s::STD_MATH_MIN | s::STD_MATH_MAX => check_min_max(c, name, args, span),
        _ => return None,
    };
    Some(ty)
}

fn check_abs(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 1 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 1 argument, got {}",
                s::STD_MATH_ABS,
                args.len()
            ),
            "Example: Std.Math.Abs(-5) or Std.Math.Abs(-3.5).",
            span,
        );
        return Ty::Error;
    }
    let ty = c.check_expr(&args[0]);
    if ty == Ty::Integer {
        Ty::Integer
    } else if ty == Ty::Real {
        Ty::Real
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects integer or real", s::STD_MATH_ABS),
            "Use a numeric argument.",
            span,
        );
        Ty::Error
    }
}

fn check_min_max(c: &mut Checker, name: &str, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{name}` expects 2 arguments, got {}", args.len()),
            "Example: Std.Math.Min(A, B).",
            span,
        );
        return Ty::Error;
    }
    let left_ty = c.check_expr(&args[0]);
    let right_ty = c.check_expr(&args[1]);
    if !left_ty.is_numeric() || !right_ty.is_numeric() {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{name}` expects two numeric arguments"),
            "Use integers or reals.",
            span,
        );
        return Ty::Error;
    }
    match (&left_ty, &right_ty) {
        (Ty::Integer, Ty::Integer) => Ty::Integer,
        (Ty::Real, Ty::Real) => Ty::Real,
        _ => Ty::Real,
    }
}
