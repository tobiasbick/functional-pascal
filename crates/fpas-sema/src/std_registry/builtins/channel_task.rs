//! Type-checking builtins for `Std.Channel` and `Std.Task`.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md` (from the repository root).

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

pub(super) fn check_channel_task_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    let ty = match name {
        s::STD_CHANNEL_MAKE => check_channel_make(c, name, args, span),
        s::STD_CHANNEL_MAKE_BUFFERED => check_channel_make_buffered(c, args, span),
        s::STD_CHANNEL_SEND => check_channel_send(c, args, span),
        s::STD_CHANNEL_RECEIVE => check_channel_receive(c, args, span),
        s::STD_CHANNEL_TRY_RECEIVE => check_channel_try_receive(c, args, span),
        s::STD_CHANNEL_CLOSE => check_channel_close(c, args, span),
        s::STD_TASK_WAIT => check_task_wait(c, args, span),
        s::STD_TASK_WAIT_ALL => check_task_wait_all(c, args, span),
        _ => return None,
    };
    Some(ty)
}

fn expect_args(c: &mut Checker, name: &str, args: &[Expr], expected: usize, span: Span) -> bool {
    if args.len() == expected {
        return true;
    }

    c.error_with_code(
        SEMA_WRONG_ARGUMENT_COUNT,
        format!(
            "`{name}` expects {expected} argument(s), got {}",
            args.len()
        ),
        format!(
            "Call `{name}` with exactly {expected} argument{}.",
            if expected == 1 { "" } else { "s" }
        ),
        span,
    );
    false
}

fn expect_channel_arg(c: &mut Checker, expr: &Expr, context: &str) -> Option<Ty> {
    let channel_ty = c.check_expr(expr);
    match channel_ty {
        Ty::Channel(inner) => Some(*inner),
        Ty::Error => Some(Ty::Error),
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in {context}: expected `Channel(Error)`, found `{other:?}`"),
                "Pass a channel created with `Std.Channel.Make` or `Std.Channel.MakeBuffered`.",
                crate::check::spans::expr_span(expr),
            );
            None
        }
    }
}

fn expect_task_arg(c: &mut Checker, expr: &Expr, context: &str) -> Option<Ty> {
    let task_ty = c.check_expr(expr);
    match task_ty {
        Ty::Task(inner) => Some(*inner),
        Ty::Error => Some(Ty::Error),
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in {context}: expected `Task(Error)`, found `{other:?}`"),
                "Pass a task handle produced by `go FunctionName(args)`.",
                crate::check::spans::expr_span(expr),
            );
            None
        }
    }
}

fn check_channel_make(c: &mut Checker, name: &str, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, name, args, 0, span) {
        return Ty::Error;
    }
    Ty::Channel(Box::new(Ty::Error))
}

fn check_channel_make_buffered(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_CHANNEL_MAKE_BUFFERED, args, 1, span) {
        return Ty::Error;
    }

    let size_ty = c.check_expr(&args[0]);
    c.check_type_compat(
        &Ty::Integer,
        &size_ty,
        "channel buffer size",
        crate::check::spans::expr_span(&args[0]),
    );

    Ty::Channel(Box::new(Ty::Error))
}

fn check_channel_send(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_CHANNEL_SEND, args, 2, span) {
        return Ty::Error;
    }

    let value_ty = c.check_expr(&args[1]);
    if let Some(inner_ty) = expect_channel_arg(c, &args[0], "channel send target")
        && !inner_ty.is_error()
    {
        c.check_type_compat(
            &inner_ty,
            &value_ty,
            "channel send value",
            crate::check::spans::expr_span(&args[1]),
        );
    }

    Ty::Unit
}

fn check_channel_receive(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_CHANNEL_RECEIVE, args, 1, span) {
        return Ty::Error;
    }

    expect_channel_arg(c, &args[0], "channel receive source").unwrap_or(Ty::Error)
}

fn check_channel_try_receive(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_CHANNEL_TRY_RECEIVE, args, 1, span) {
        return Ty::Error;
    }

    Ty::Option(Box::new(
        expect_channel_arg(c, &args[0], "channel receive source").unwrap_or(Ty::Error),
    ))
}

fn check_channel_close(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_CHANNEL_CLOSE, args, 1, span) {
        return Ty::Error;
    }

    let _ = expect_channel_arg(c, &args[0], "channel close target");
    Ty::Unit
}

fn check_task_wait(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_TASK_WAIT, args, 1, span) {
        return Ty::Error;
    }

    expect_task_arg(c, &args[0], "task wait target").unwrap_or(Ty::Error)
}

fn check_task_wait_all(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_TASK_WAIT_ALL, args, 1, span) {
        return Ty::Error;
    }

    let tasks_ty = c.check_expr(&args[0]);
    match tasks_ty {
        Ty::Array(inner) if matches!(inner.as_ref(), Ty::Task(_) | Ty::Error) => Ty::Unit,
        Ty::Array(inner) => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Type mismatch in task list: expected `Array(Task(Error))`, found `Array({inner:?})`"
                ),
                "Pass an array of task handles such as `[T1, T2, T3]`.",
                crate::check::spans::expr_span(&args[0]),
            );
            Ty::Unit
        }
        Ty::Error => Ty::Unit,
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Type mismatch in task list: expected `Array(Task(Error))`, found `{other:?}`"
                ),
                "Pass an array of task handles such as `[T1, T2, T3]`.",
                crate::check::spans::expr_span(&args[0]),
            );
            Ty::Unit
        }
    }
}
