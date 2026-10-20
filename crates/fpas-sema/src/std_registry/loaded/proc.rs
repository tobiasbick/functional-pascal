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
    let environment = type_registration::register_record_type(
        checker,
        s::STD_PROC_ENVIRONMENT_VARIABLE,
        vec![
            ("Name".into(), Ty::String),
            ("Value".into(), Ty::Option(Box::new(Ty::String))),
        ],
    );
    let token = type_registration::register_record_type(
        checker,
        s::STD_TASK_CANCELLATION_TOKEN,
        Vec::new(),
    );
    let options = type_registration::register_record_type(
        checker,
        s::STD_PROC_PROCESS_OPTIONS,
        vec![
            ("WorkingDirectory".into(), Ty::Option(Box::new(Ty::String))),
            ("Environment".into(), Ty::Array(Box::new(environment))),
            ("Input".into(), Ty::String),
            ("TimeoutMillis".into(), Ty::Option(Box::new(Ty::Integer))),
            ("MaxOutputBytes".into(), Ty::Integer),
            ("CancellationToken".into(), Ty::Option(Box::new(token))),
        ],
    );
    define_func(
        checker,
        s::STD_PROC_DEFAULT_OPTIONS,
        vec![],
        options.clone(),
    );
    for (name, output) in [
        (s::STD_PROC_RUN_WITH_OPTIONS, Ty::Integer),
        (s::STD_PROC_RUN_CAPTURE_WITH_OPTIONS, process_output.clone()),
    ] {
        define_func(
            checker,
            name,
            vec![
                p("Command", Ty::String),
                p("Args", Ty::Array(Box::new(Ty::String))),
                p("Options", options.clone()),
            ],
            Ty::Result(Box::new(output), Box::new(error_ty.clone())),
        );
    }
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
