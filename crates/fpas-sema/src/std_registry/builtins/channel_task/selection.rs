//! Typed delivery callbacks for `Std.Task` selection cases.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use super::*;
use crate::types::{ParamTy, ProcedureTy};

/// Check each source's delivery type before returning the common opaque case type.
pub(super) fn check_case(c: &mut Checker, name: &str, args: &[Expr], span: Span) -> Ty {
    let count = if name == s::STD_TASK_SEND_CASE { 3 } else { 2 };
    if !expect_args(c, name, args, count, span) {
        return Ty::Error;
    }
    let payload = match name {
        s::STD_TASK_RECEIVE_CASE => Some(expect_channel_arg(c, &args[0]).unwrap_or(Ty::Error)),
        s::STD_TASK_SEND_CASE => {
            check_send(c, &args[..2], span, name, ChannelWaitArg::None);
            Some(Ty::Boolean)
        }
        s::STD_TASK_TASK_CASE => {
            expect_task_arg(c, &args[0], "task completion case");
            None
        }
        s::STD_TASK_TIMER_CASE => {
            expect_type(c, &args[0], &Ty::Integer, "timer case milliseconds");
            None
        }
        s::STD_TASK_CANCELLATION_CASE => {
            expect_cancellation_token(c, &args[0]);
            None
        }
        _ => unreachable!("selection constructor dispatch"),
    };
    let params = payload
        .into_iter()
        .map(|ty| ParamTy {
            name: "Outcome".into(),
            mutable: false,
            ty: Ty::Result(Box::new(ty), Box::new(Ty::String)),
        })
        .collect();
    expect_type(
        c,
        &args[count - 1],
        &Ty::Procedure(ProcedureTy {
            type_params: vec![],
            params,
            variadic: false,
        }),
        "selection callback",
    );
    c.scopes
        .lookup(s::STD_TASK_WAIT_CASE)
        .map_or(Ty::Error, |symbol| symbol.ty.clone())
}
