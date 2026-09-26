//! Registration of `Std.Task`.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md` (from the repository root); language rules: `docs/pascal/language/concurrency/README.md`.

use super::super::{define_builtin_std, define_func, p};
use super::type_registration;
use crate::check::Checker;
use crate::types::{FunctionTy, Ty};
use fpas_std::std_symbols as s;

/// Register the Task unit's handles and task-aware intrinsic declarations.
pub fn register_std_task(c: &mut Checker) {
    let case = type_registration::register_record_type(c, s::STD_TASK_WAIT_CASE, Vec::new());
    define_func(
        c,
        s::STD_TASK_SELECT,
        vec![p("Cases", Ty::Array(Box::new(case.clone())), false)],
        Ty::Integer,
    );
    define_func(
        c,
        s::STD_TASK_CLOSE_WAIT_CASE,
        vec![p("Handle", case, false)],
        Ty::Boolean,
    );
    let source =
        type_registration::register_record_type(c, s::STD_TASK_CANCELLATION_SOURCE, Vec::new());
    let token =
        type_registration::register_record_type(c, s::STD_TASK_CANCELLATION_TOKEN, Vec::new());
    let group = type_registration::register_record_type(c, s::STD_TASK_TASK_GROUP, Vec::new());
    let kind = type_registration::register_enum_type(
        c,
        s::STD_TASK_TASK_FAILURE_KIND,
        &["ReturnedError", "Panicked", "RuntimeError", "Cancelled"],
    );
    let failure = type_registration::register_record_type(
        c,
        s::STD_TASK_TASK_FAILURE,
        vec![
            ("TaskId".into(), Ty::Integer),
            ("Kind".into(), kind),
            ("Message".into(), Ty::String),
            ("Code".into(), Ty::Integer),
            ("Line".into(), Ty::Integer),
            ("Column".into(), Ty::Integer),
        ],
    );
    define_func(c, s::STD_TASK_CREATE_TASK_GROUP, vec![], group.clone());
    define_func(
        c,
        s::STD_TASK_GET_TASK_GROUP_TOKEN,
        vec![p("Group", group.clone(), false)],
        token.clone(),
    );
    define_func(
        c,
        s::STD_TASK_CANCEL_TASK_GROUP,
        vec![p("Group", group.clone(), false)],
        Ty::Boolean,
    );
    define_func(
        c,
        s::STD_TASK_CLOSE_TASK_GROUP,
        vec![p("Group", group.clone(), false)],
        Ty::Array(Box::new(failure.clone())),
    );
    define_func(
        c,
        s::STD_TASK_CLOSE_TASK_GROUP_WITH_TIMEOUT,
        vec![
            p("Group", group.clone(), false),
            p("TimeoutMillis", Ty::Integer, false),
        ],
        Ty::Result(
            Box::new(Ty::Array(Box::new(failure.clone()))),
            Box::new(Ty::String),
        ),
    );
    define_func(
        c,
        s::STD_TASK_TRY_CLOSE_COMPLETED_TASK_GROUP,
        vec![p("Group", group, false)],
        Ty::Option(Box::new(Ty::Array(Box::new(failure)))),
    );

    define_func(
        c,
        s::STD_TASK_CREATE_CANCELLATION_SOURCE,
        vec![],
        source.clone(),
    );
    define_func(
        c,
        s::STD_TASK_GET_CANCELLATION_TOKEN,
        vec![p("Source", source.clone(), false)],
        token.clone(),
    );
    define_func(
        c,
        s::STD_TASK_CANCEL,
        vec![p("Source", source, false)],
        Ty::Boolean,
    );
    define_func(
        c,
        s::STD_TASK_IS_CANCELLATION_REQUESTED,
        vec![p("Token", token, false)],
        Ty::Boolean,
    );

    let placeholder = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![],
        return_type: Box::new(Ty::Error),
        variadic: false,
    });

    for name in [
        s::STD_TASK_START_TASK_IN_GROUP,
        s::STD_TASK_START_SUPERVISED_TASK,
        s::STD_TASK_RECEIVE_CASE,
        s::STD_TASK_SEND_CASE,
        s::STD_TASK_TASK_CASE,
        s::STD_TASK_TIMER_CASE,
        s::STD_TASK_CANCELLATION_CASE,
        s::STD_TASK_CREATE_CHANNEL,
        s::STD_TASK_SEND,
        s::STD_TASK_TRY_SEND,
        s::STD_TASK_SEND_WITH_CANCELLATION,
        s::STD_TASK_SEND_WITH_TIMEOUT,
        s::STD_TASK_RECEIVE,
        s::STD_TASK_TRY_RECEIVE,
        s::STD_TASK_RECEIVE_WITH_CANCELLATION,
        s::STD_TASK_RECEIVE_WITH_TIMEOUT,
        s::STD_TASK_CLOSE_CHANNEL,
        s::STD_TASK_WAIT,
        s::STD_TASK_WAIT_ALL,
        s::STD_TASK_WAIT_ANY,
        s::STD_TASK_WAIT_ANY_WITH_TIMEOUT,
        s::STD_TASK_WAIT_ANY_WITH_CANCELLATION,
    ] {
        define_builtin_std(c, name, placeholder.clone());
    }
}
