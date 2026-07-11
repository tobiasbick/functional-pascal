use super::super::super::p;
use super::TuiCallbackTypes;
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_std::std_symbols as s;

/// Register Turbo Vision callback signatures and the `ApplicationHandlers` record.
pub(super) fn register_application_handlers(
    checker: &mut Checker,
    application: &Ty,
    key_event: &Ty,
    console_event: &Ty,
) -> (Ty, TuiCallbackTypes) {
    let on_command = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
        variadic: false,
    });
    let on_key = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("Key", key_event.clone(), false),
        ],
        return_type: Box::new(Ty::Boolean),
        variadic: false,
    });
    let on_mouse = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("Event", console_event.clone(), false),
        ],
        variadic: false,
    });

    let application_handlers = type_registration::register_record_type_with_defaults(
        checker,
        s::STD_TUI_APPLICATION_HANDLERS,
        vec![
            ("OnCommand".into(), on_command.clone()),
            ("OnKey".into(), Ty::Option(Box::new(on_key.clone()))),
            ("OnMouse".into(), Ty::Option(Box::new(on_mouse.clone()))),
        ],
        vec![
            ("OnCommand".into(), None),
            ("OnKey".into(), Some(type_registration::default_none_expr())),
            (
                "OnMouse".into(),
                Some(type_registration::default_none_expr()),
            ),
        ],
    );

    let callbacks = TuiCallbackTypes { on_key, on_mouse };

    (application_handlers, callbacks)
}
