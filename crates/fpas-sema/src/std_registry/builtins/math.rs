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
        s::STD_MATH_SIGN => check_sign(c, args, span),
        s::STD_MATH_CLAMP => check_clamp(c, args, span),
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
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{name}` requires both arguments to be the same numeric kind (both integer or both real)"),
                "Use explicit conversion, for example `IntToReal(N)`, to match types.",
                span,
            );
            Ty::Error
        }
    }
}

fn check_sign(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 1 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 1 argument, got {}",
                s::STD_MATH_SIGN,
                args.len()
            ),
            "Example: Std.Math.Sign(-5) or Std.Math.Sign(-3.5).",
            span,
        );
        return Ty::Error;
    }
    let ty = c.check_expr(&args[0]);
    if ty == Ty::Integer || ty == Ty::Real {
        Ty::Integer
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects integer or real", s::STD_MATH_SIGN),
            "Use a numeric argument.",
            span,
        );
        Ty::Error
    }
}

fn check_clamp(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 3 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 3 arguments, got {}",
                s::STD_MATH_CLAMP,
                args.len()
            ),
            "Example: Std.Math.Clamp(X, Lo, Hi).",
            span,
        );
        return Ty::Error;
    }
    let x_ty = c.check_expr(&args[0]);
    let lo_ty = c.check_expr(&args[1]);
    let hi_ty = c.check_expr(&args[2]);
    if !x_ty.is_numeric() || !lo_ty.is_numeric() || !hi_ty.is_numeric() {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects numeric arguments", s::STD_MATH_CLAMP),
            "Use integers or reals.",
            span,
        );
        return Ty::Error;
    }
    match (&x_ty, &lo_ty, &hi_ty) {
        (Ty::Integer, Ty::Integer, Ty::Integer) => Ty::Integer,
        (Ty::Real, Ty::Real, Ty::Real) => Ty::Real,
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{}` requires all three arguments to be the same numeric kind (all integer or all real)",
                    s::STD_MATH_CLAMP
                ),
                "Use explicit conversion, for example `IntToReal(N)`, to match types.",
                span,
            );
            Ty::Error
        }
    }
}
