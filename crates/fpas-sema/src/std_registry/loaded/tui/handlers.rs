use super::super::super::p;
use super::TuiCallbackTypes;
use crate::types::{FunctionTy, ProcedureTy, Ty};

/// Register Turbo Vision callback signatures for `Application.OnCommand`, `OnKey`, and `OnMouse`.
pub(super) fn register_turbo_vision_callbacks(
    application: &Ty,
    key_event: &Ty,
    console_event: &Ty,
) -> TuiCallbackTypes {
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
    let on_command = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
        variadic: false,
    });

    TuiCallbackTypes {
        on_command,
        on_key,
        on_mouse,
    }
}
