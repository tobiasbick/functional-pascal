use super::super::{define_func, p};
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_builtins::std_symbols as s;

/// Register the `Std.Os.Proc` functions made visible by `uses Std.Os.Proc`.
pub(super) fn register_std_proc(checker: &mut Checker) {
    let error_ty = super::errors::register(
        checker,
        fpas_builtins::native_errors::NativeErrorFamily::Proc,
    );
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
            (
                "Environment".into(),
                Ty::Array(Box::new(environment.clone())),
            ),
            ("Input".into(), Ty::String),
            ("TimeoutMillis".into(), Ty::Option(Box::new(Ty::Integer))),
            ("MaxOutputBytes".into(), Ty::Integer),
            (
                "CancellationToken".into(),
                Ty::Option(Box::new(token.clone())),
            ),
        ],
    );
    let process = type_registration::register_opaque_type(checker, s::STD_PROC_PROCESS);
    let exit_status = type_registration::register_record_type(
        checker,
        s::STD_PROC_EXIT_STATUS,
        vec![
            ("ExitCode".into(), Ty::Option(Box::new(Ty::Integer))),
            ("Signal".into(), Ty::Option(Box::new(Ty::String))),
        ],
    );
    let start_options = type_registration::register_record_type(
        checker,
        s::STD_PROC_START_OPTIONS,
        vec![
            ("WorkingDirectory".into(), Ty::Option(Box::new(Ty::String))),
            ("Environment".into(), Ty::Array(Box::new(environment))),
        ],
    );
    let result = |ty| Ty::Result(Box::new(ty), Box::new(error_ty.clone()));
    define_func(
        checker,
        s::STD_PROC_DEFAULT_START_OPTIONS,
        vec![],
        start_options.clone(),
    );
    define_func(
        checker,
        s::STD_PROC_START,
        vec![
            p("Command", Ty::String),
            p("Args", Ty::Array(Box::new(Ty::String))),
            p("Options", start_options.clone()),
        ],
        result(process.clone()),
    );
    for name in [s::STD_PROC_READ_STDOUT, s::STD_PROC_READ_STDERR] {
        define_func(
            checker,
            name,
            vec![p("Process", process.clone()), p("MaxBytes", Ty::Integer)],
            result(Ty::Option(Box::new(Ty::Array(Box::new(Ty::Integer))))),
        );
    }
    define_func(
        checker,
        s::STD_PROC_WRITE_STDIN,
        vec![
            p("Process", process.clone()),
            p("Bytes", Ty::Array(Box::new(Ty::Integer))),
        ],
        result(Ty::Integer),
    );
    for name in [
        s::STD_PROC_CLOSE_STDIN,
        s::STD_PROC_TERMINATE,
        s::STD_PROC_CLOSE,
    ] {
        define_func(
            checker,
            name,
            vec![p("Process", process.clone())],
            result(Ty::Boolean),
        );
    }
    define_func(
        checker,
        s::STD_PROC_TRY_WAIT_STATUS,
        vec![p("Process", process.clone())],
        result(Ty::Option(Box::new(exit_status.clone()))),
    );
    define_func(
        checker,
        s::STD_PROC_RUN_INTERACTIVE,
        vec![
            p("Command", Ty::String),
            p("Args", Ty::Array(Box::new(Ty::String))),
            p("Options", start_options),
            p("TimeoutMillis", Ty::Option(Box::new(Ty::Integer))),
            p("CancellationToken", Ty::Option(Box::new(token))),
        ],
        result(exit_status),
    );
    define_func(
        checker,
        s::STD_PROC_TRY_WAIT,
        vec![p("Process", process)],
        result(Ty::Option(Box::new(Ty::Integer))),
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
