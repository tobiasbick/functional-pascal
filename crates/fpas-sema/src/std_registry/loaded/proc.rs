use super::super::{define_func, p};
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the `Std.Proc` functions made visible by `uses Std.Proc`.
pub(super) fn register_std_proc(checker: &mut Checker) {
    let error_ty =
        super::errors::register(checker, fpas_std::native_errors::NativeErrorFamily::Proc);
    let process_output = type_registration::register_record_type(
        checker,
        s::STD_PROC_PROCESS_OUTPUT,
        vec![
            ("ExitCode".into(), Ty::Integer),
            ("Stdout".into(), Ty::String),
            ("Stderr".into(), Ty::String),
        ],
    );
    define_func(
        checker,
        s::STD_PROC_CURRENT_EXECUTABLE,
        vec![],
        Ty::Result(Box::new(Ty::String), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_PROC_RUN,
        vec![
            p("Command", Ty::String),
            p("Args", Ty::Array(Box::new(Ty::String))),
        ],
        Ty::Result(Box::new(Ty::Integer), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_PROC_RUN_CAPTURE,
        vec![
            p("Command", Ty::String),
            p("Args", Ty::Array(Box::new(Ty::String))),
        ],
        Ty::Result(Box::new(process_output), Box::new(error_ty.clone())),
    );
}
