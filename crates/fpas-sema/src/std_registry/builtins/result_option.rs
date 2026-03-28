//! Type-checking builtins for `Std.Result` and `Std.Option` calls.
//!
//! **Documentation:** `docs/pascal/std/result.md` and `docs/pascal/std/option.md` (from the repository root).

use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
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
        s::STD_RESULT_MAP => check_result_map(c, args, span),
        s::STD_RESULT_AND_THEN => check_result_and_then(c, args, span),
        s::STD_RESULT_OR_ELSE => check_result_or_else(c, args, span),
        s::STD_OPTION_UNWRAP => check_one_arg(c, name, args, span, unwrap_option),
        s::STD_OPTION_UNWRAP_OR => check_two_args(c, name, args, span, |ty, _| unwrap_option(ty)),
        s::STD_OPTION_IS_SOME | s::STD_OPTION_IS_NONE => {
            check_one_arg(c, name, args, span, |_| Ty::Boolean)
        }
        s::STD_OPTION_MAP => check_option_map(c, args, span),
        s::STD_OPTION_AND_THEN => check_option_and_then(c, args, span),
        s::STD_OPTION_OR_ELSE => check_option_or_else(c, args, span),
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

/// `Std.Result.Map(R, F)` -> `Result of U, E` where `F: function(V: T): U`.
fn check_result_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{}` expects 2 arguments, got {}", s::STD_RESULT_MAP, args.len()),
            "Example: Std.Result.Map(R, function(V: integer): string begin return IntToStr(V) end).",
            span,
        );
        return Ty::Error;
    }
    let result_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let err_ty = match &result_ty {
        Ty::Result(_, e) => *e.clone(),
        _ => Ty::Error,
    };
    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => {
            Ty::Result(return_type.clone(), Box::new(err_ty))
        }
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{}` second argument must be a function", s::STD_RESULT_MAP),
                "Pass a function(V: T): U.",
                span,
            );
            Ty::Error
        }
    }
}

/// `Std.Result.AndThen(R, F)` -> `Result of U, E` where `F: function(V: T): Result of U, E`.
fn check_result_and_then(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{}` expects 2 arguments, got {}", s::STD_RESULT_AND_THEN, args.len()),
            "Example: Std.Result.AndThen(R, function(V: integer): Result of string, string begin return Ok(IntToStr(V)) end).",
            span,
        );
        return Ty::Error;
    }
    let _result_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => *return_type.clone(),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{}` second argument must be a function",
                    s::STD_RESULT_AND_THEN
                ),
                "Pass a function(V: T): Result of U, E.",
                span,
            );
            Ty::Error
        }
    }
}

/// `Std.Result.OrElse(R, F)` -> `Result of T, F` where `F: function(E: E): Result of T, F`.
fn check_result_or_else(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{}` expects 2 arguments, got {}", s::STD_RESULT_OR_ELSE, args.len()),
            "Example: Std.Result.OrElse(R, function(E: string): Result of integer, string begin return Ok(0) end).",
            span,
        );
        return Ty::Error;
    }
    let _result_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => *return_type.clone(),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{}` second argument must be a function",
                    s::STD_RESULT_OR_ELSE
                ),
                "Pass a function(E: E): Result of T, F.",
                span,
            );
            Ty::Error
        }
    }
}

/// `Std.Option.Map(O, F)` -> `Option of U` where `F: function(V: T): U`.
fn check_option_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{}` expects 2 arguments, got {}", s::STD_OPTION_MAP, args.len()),
            "Example: Std.Option.Map(O, function(V: integer): string begin return IntToStr(V) end).",
            span,
        );
        return Ty::Error;
    }
    let _opt_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => Ty::Option(return_type.clone()),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{}` second argument must be a function", s::STD_OPTION_MAP),
                "Pass a function(V: T): U.",
                span,
            );
            Ty::Error
        }
    }
}

/// `Std.Option.AndThen(O, F)` -> `Option of U` where `F: function(V: T): Option of U`.
fn check_option_and_then(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{}` expects 2 arguments, got {}", s::STD_OPTION_AND_THEN, args.len()),
            "Example: Std.Option.AndThen(O, function(V: integer): Option of string begin return Some(IntToStr(V)) end).",
            span,
        );
        return Ty::Error;
    }
    let _opt_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => *return_type.clone(),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{}` second argument must be a function",
                    s::STD_OPTION_AND_THEN
                ),
                "Pass a function(V: T): Option of U.",
                span,
            );
            Ty::Error
        }
    }
}

/// `Std.Option.OrElse(O, F)` -> `Option of T` where `F: function(): Option of T`.
fn check_option_or_else(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!("`{}` expects 2 arguments, got {}", s::STD_OPTION_OR_ELSE, args.len()),
            "Example: Std.Option.OrElse(O, function(): Option of integer begin return Some(0) end).",
            span,
        );
        return Ty::Error;
    }
    let _opt_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    match &func_ty {
        Ty::Function(FunctionTy { return_type, .. }) => *return_type.clone(),
        _ => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{}` second argument must be a function",
                    s::STD_OPTION_OR_ELSE
                ),
                "Pass a function(): Option of T.",
                span,
            );
            Ty::Error
        }
    }
}
