//! Typed task creation within an explicit lifecycle owner.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use super::*;

/// Preserve the child's return type while checking its cancellation parameter and captures.
pub(super) fn check_start(c: &mut Checker, name: &str, args: &[&Expr], span: Span) -> Ty {
    let supervised = name == s::STD_TASK_START_SUPERVISED_TASK;
    if !expect_args(c, name, args, if supervised { 4 } else { 2 }, span) {
        return Ty::Error;
    }
    if supervised {
        expect_type(c, args[2], &Ty::Integer, "retry limit");
        expect_type(c, args[3], &Ty::Integer, "retry backoff");
    }
    let group = c
        .scopes
        .lookup(s::STD_TASK_TASK_GROUP)
        .map_or(Ty::Error, |s| s.ty.clone());
    expect_type(c, args[0], &group, "task group");
    let work = c.check_expr(args[1]);
    if c.expr_is_task_bound(crate::expr_lookup_key(args[1])) {
        c.error_with_code(
            SEMA_TASK_BOUND_CALLABLE,
            "Cannot start a task-bound worker in another task",
            "Use immutable captures for group workers.",
            args[1].span(),
        );
    }
    let (params, result) = match work {
        Ty::Function(work) => (work.params, *work.return_type),
        Ty::Procedure(work) => (work.params, Ty::Unit),
        Ty::Error => return Ty::Error,
        other => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Expected a worker routine, found `{other}`"),
                "Pass a routine accepting one CancellationToken.",
                args[1].span(),
            );
            return Ty::Error;
        }
    };
    let token = c
        .scopes
        .lookup(s::STD_TASK_CANCELLATION_TOKEN)
        .map_or(Ty::Error, |s| s.ty.clone());
    if params.len() != 1 || !params[0].ty.compatible_with(&token) {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "A group worker must accept exactly one CancellationToken",
            "Declare Work(Token: CancellationToken).",
            args[1].span(),
        );
    }
    if let Ty::Result(_, error) = &result
        && !matches!(error.as_ref(), Ty::String | Ty::Error)
    {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            "A group worker's Result error type must be string",
            "Use result of T, string for an ordinary worker error.",
            args[1].span(),
        );
    }
    Ty::Task(Box::new(result))
}
