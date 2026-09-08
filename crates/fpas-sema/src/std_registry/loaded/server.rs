//! Server lifetime signatures. See `docs/pascal/std/network/server.md`.

use super::super::{define_func, p};
use super::type_registration::register_record_type;
use crate::{check::Checker, types::Ty};
use fpas_std::std_symbols as s;

/// Register the opaque lifetime and its hosted operations.
pub(super) fn register(checker: &mut Checker) {
    let lifetime = register_record_type(checker, s::STD_SERVER_LIFETIME, vec![]);
    let group = register_record_type(checker, s::STD_TASK_TASK_GROUP, vec![]);
    let token = register_record_type(checker, s::STD_TASK_CANCELLATION_TOKEN, vec![]);
    let listener = register_record_type(checker, s::STD_NET_LISTENER, vec![]);
    let result = |ty| Ty::Result(Box::new(ty), Box::new(Ty::String));
    define_func(
        checker,
        s::STD_SERVER_CREATE_LIFETIME,
        vec![
            p("GraceMillis", Ty::Integer, false),
            p("ForceExit", Ty::Boolean, false),
        ],
        result(lifetime.clone()),
    );
    for (name, ty) in [
        (s::STD_SERVER_GET_WORK_GROUP, group),
        (s::STD_SERVER_GET_STOP_TOKEN, token),
        (s::STD_SERVER_IS_READY, Ty::Boolean),
        (s::STD_SERVER_REQUEST_STOP, Ty::Boolean),
        (s::STD_SERVER_REMAINING_MILLIS, Ty::Integer),
        (s::STD_SERVER_FINISH_SHUTDOWN, result(Ty::Boolean)),
        (s::STD_SERVER_OBSERVE_SIGNALS, result(Ty::Boolean)),
        (
            s::STD_SERVER_SHUTDOWN_ERRORS,
            Ty::Array(Box::new(Ty::String)),
        ),
    ] {
        define_func(
            checker,
            name,
            vec![p("Lifetime", lifetime.clone(), false)],
            ty,
        );
    }
    define_func(
        checker,
        s::STD_SERVER_OWN_LISTENER,
        vec![
            p("Lifetime", lifetime, false),
            p("Listener", listener, false),
        ],
        result(Ty::Boolean),
    );
}
