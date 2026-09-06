//! Type-checking builtins for `Std.Task`.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root); language rules: `docs/pascal/language/concurrency/README.md`.

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_TASK_BOUND_CALLABLE, SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT,
};
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
        s::STD_TASK_CREATE_CHANNEL => check_create_channel(c, args, span),
        s::STD_TASK_SEND | s::STD_TASK_TRY_SEND => {
            check_send(c, args, span, name, ChannelWaitArg::None)
        }
        s::STD_TASK_SEND_WITH_CANCELLATION => {
            check_send(c, args, span, name, ChannelWaitArg::Cancellation)
        }
        s::STD_TASK_SEND_WITH_TIMEOUT => check_send(c, args, span, name, ChannelWaitArg::Timeout),
        s::STD_TASK_RECEIVE => check_receive(c, args, span, name, ChannelWaitArg::None, false),
        s::STD_TASK_TRY_RECEIVE => check_receive(c, args, span, name, ChannelWaitArg::None, true),
        s::STD_TASK_RECEIVE_WITH_CANCELLATION => {
            check_receive(c, args, span, name, ChannelWaitArg::Cancellation, false)
        }
        s::STD_TASK_RECEIVE_WITH_TIMEOUT => {
            check_receive(c, args, span, name, ChannelWaitArg::Timeout, false)
        }
        s::STD_TASK_CLOSE_CHANNEL => check_close_channel(c, args, span),
        s::STD_TASK_WAIT => check_task_wait(c, args, span),
        s::STD_TASK_WAIT_ALL => check_task_wait_all(c, args, span, s::STD_TASK_WAIT_ALL),
        s::STD_TASK_WAIT_ANY => {
            check_task_wait_all(c, args, span, s::STD_TASK_WAIT_ANY);
            Ty::Integer
        }
        s::STD_TASK_WAIT_ANY_WITH_TIMEOUT | s::STD_TASK_WAIT_ANY_WITH_CANCELLATION => {
            if !expect_args(c, name, args, 2, span) {
                return Some(Ty::Error);
            }
            check_task_wait_all(c, &args[..1], span, name);
            if name == s::STD_TASK_WAIT_ANY_WITH_TIMEOUT {
                expect_type(c, &args[1], &Ty::Integer, "task wait timeout");
            } else {
                expect_cancellation_token(c, &args[1]);
            }
            Ty::Result(Box::new(Ty::Integer), Box::new(Ty::String))
        }
        _ => return None,
    };
    Some(ty)
}

#[derive(Clone, Copy)]
enum ChannelWaitArg {
    None,
    Cancellation,
    Timeout,
}

fn check_create_channel(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_TASK_CREATE_CHANNEL, args, 1, span) {
        return Ty::Error;
    }
    expect_type(c, &args[0], &Ty::Integer, "channel capacity");
    Ty::Channel(Box::new(Ty::Error))
}

fn check_send(
    c: &mut Checker,
    args: &[Expr],
    span: Span,
    name: &str,
    wait_arg: ChannelWaitArg,
) -> Ty {
    let expected = match wait_arg {
        ChannelWaitArg::None => 2,
        ChannelWaitArg::Cancellation | ChannelWaitArg::Timeout => 3,
    };
    if !expect_args(c, name, args, expected, span) {
        return Ty::Error;
    }

    let channel = expect_channel_arg(c, &args[0]);
    let value = c.check_expr(&args[1]);
    if c.expr_is_task_bound(crate::expr_lookup_key(&args[1])) {
        c.error_with_code(
            SEMA_TASK_BOUND_CALLABLE,
            "Cannot send a task-bound value through a channel",
            "Mutable captures make a value task-bound. Send immutable data or a callable with immutable captures instead.",
            args[1].span(),
        );
    }
    if let Some(element) = &channel
        && !element.compatible_with(&value)
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("Type mismatch in channel send: expected `{element}`, found `{value}`"),
            "Send a value matching the channel element type.",
            args[1].span(),
        );
    }
    match wait_arg {
        ChannelWaitArg::None => {}
        ChannelWaitArg::Cancellation => expect_cancellation_token(c, &args[2]),
        ChannelWaitArg::Timeout => expect_type(c, &args[2], &Ty::Integer, "channel send timeout"),
    }
    channel_result(Ty::Boolean)
}

fn check_receive(
    c: &mut Checker,
    args: &[Expr],
    span: Span,
    name: &str,
    wait_arg: ChannelWaitArg,
    optional: bool,
) -> Ty {
    let expected = match wait_arg {
        ChannelWaitArg::None => 1,
        ChannelWaitArg::Cancellation | ChannelWaitArg::Timeout => 2,
    };
    if !expect_args(c, name, args, expected, span) {
        return Ty::Error;
    }
    let mut element = expect_channel_arg(c, &args[0]).unwrap_or(Ty::Error);
    match wait_arg {
        ChannelWaitArg::None => {}
        ChannelWaitArg::Cancellation => expect_cancellation_token(c, &args[1]),
        ChannelWaitArg::Timeout => {
            expect_type(c, &args[1], &Ty::Integer, "channel receive timeout")
        }
    }
    if optional {
        element = Ty::Option(Box::new(element));
    }
    channel_result(element)
}

fn check_close_channel(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_TASK_CLOSE_CHANNEL, args, 1, span) {
        return Ty::Error;
    }
    expect_channel_arg(c, &args[0]);
    Ty::Boolean
}

fn channel_result(value: Ty) -> Ty {
    Ty::Result(Box::new(value), Box::new(Ty::String))
}

fn expect_channel_arg(c: &mut Checker, expr: &Expr) -> Option<Ty> {
    match c.check_expr(expr) {
        Ty::Channel(inner) => Some(*inner),
        Ty::Error => Some(Ty::Error),
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in channel operation: expected a channel, found `{other}`"),
                "Pass a value declared with `channel of T`.",
                expr.span(),
            );
            None
        }
    }
}

fn expect_cancellation_token(c: &mut Checker, expr: &Expr) {
    let actual = c.check_expr(expr);
    let valid = match &actual {
        Ty::Record(record) => record
            .name
            .eq_ignore_ascii_case(s::STD_TASK_CANCELLATION_TOKEN),
        Ty::Named(name) => name.eq_ignore_ascii_case(s::STD_TASK_CANCELLATION_TOKEN),
        Ty::Error => true,
        _ => false,
    };
    if !valid {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("Type mismatch in cancellable task operation: expected `Std.Task.CancellationToken`, found `{actual}`"),
            "Pass the token returned by `Std.Task.GetCancellationToken`.",
            expr.span(),
        );
    }
}

fn expect_type(c: &mut Checker, expr: &Expr, expected: &Ty, context: &str) {
    let actual = c.check_expr(expr);
    if !expected.compatible_with(&actual) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("Type mismatch in {context}: expected `{expected}`, found `{actual}`"),
            format!("Pass a value of type `{expected}`."),
            expr.span(),
        );
    }
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

fn expect_task_arg(c: &mut Checker, expr: &Expr, context: &str) -> Option<Ty> {
    let task_ty = c.check_expr(expr);
    match task_ty {
        Ty::Task(inner) => Some(*inner),
        Ty::Error => Some(Ty::Error),
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in {context}: expected a task, found `{other}`"),
                "Pass a task handle produced by `go FunctionName(args)`.",
                expr.span(),
            );
            None
        }
    }
}

fn check_task_wait(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !expect_args(c, s::STD_TASK_WAIT, args, 1, span) {
        return Ty::Error;
    }

    expect_task_arg(c, &args[0], "task wait target").unwrap_or(Ty::Error)
}

fn check_task_wait_all(c: &mut Checker, args: &[Expr], span: Span, name: &str) -> Ty {
    if !expect_args(c, name, args, 1, span) {
        return Ty::Error;
    }

    let tasks_ty = c.check_expr(&args[0]);
    match tasks_ty {
        Ty::Array(inner) if matches!(inner.as_ref(), Ty::Task(_) | Ty::Error) => Ty::Unit,
        Ty::Array(inner) => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Type mismatch in task list: expected `array of task`, found `array of {inner}`"
                ),
                "Pass an array of task handles such as `[T1, T2, T3]`.",
                args[0].span(),
            );
            Ty::Unit
        }
        Ty::Error => Ty::Unit,
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Type mismatch in task list: expected `array of task`, found `{other}`"),
                "Pass an array of task handles such as `[T1, T2, T3]`.",
                args[0].span(),
            );
            Ty::Unit
        }
    }
}
