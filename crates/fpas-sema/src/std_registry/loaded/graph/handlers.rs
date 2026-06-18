//! `Std.Graph` handler record and callback registration.
//!
//! **Documentation:** `docs/pascal/std/graph/app/README.md` (from the repository root).

use super::super::super::p;
use super::GraphCallbackTypes;
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_std::std_symbols as s;

/// Register callback signatures and the `Std.Graph.ApplicationHandlers` record.
pub(super) fn register_application_handlers(
    checker: &mut Checker,
    application_ty: &Ty,
    size_ty: &Ty,
    event_ty: &Ty,
    key_event_ty: &Ty,
    exit_reason_ty: &Ty,
) -> (Ty, GraphCallbackTypes) {
    let on_key_pressed = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("Key", key_event_ty.clone(), false),
        ],
        return_type: Box::new(Ty::Boolean),
        variadic: false,
    });
    let on_event = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("Event", event_ty.clone(), false),
        ],
        variadic: false,
    });
    let on_resize = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("NewSize", size_ty.clone(), false),
        ],
        variadic: false,
    });
    let on_paint = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application_ty.clone(), false)],
        variadic: false,
    });
    let on_close_requested = on_paint.clone();
    let on_idle = on_paint.clone();
    let on_exit = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("Reason", exit_reason_ty.clone(), false),
        ],
        variadic: false,
    });

    let application_handlers = type_registration::register_record_type_with_defaults(
        checker,
        s::STD_GRAPH_APPLICATION_HANDLERS,
        vec![
            ("OnPaint".into(), on_paint.clone()),
            (
                "OnKeyPressed".into(),
                Ty::Option(Box::new(on_key_pressed.clone())),
            ),
            ("OnMouse".into(), Ty::Option(Box::new(on_event.clone()))),
            ("OnWheel".into(), Ty::Option(Box::new(on_event.clone()))),
            ("OnResize".into(), Ty::Option(Box::new(on_resize.clone()))),
            (
                "OnCloseRequested".into(),
                Ty::Option(Box::new(on_close_requested.clone())),
            ),
            ("OnIdleMilliseconds".into(), Ty::Integer),
            ("OnIdle".into(), Ty::Option(Box::new(on_idle.clone()))),
            ("OnExit".into(), Ty::Option(Box::new(on_exit.clone()))),
        ],
        vec![
            ("OnPaint".into(), None),
            (
                "OnKeyPressed".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnMouse".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnWheel".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnResize".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnCloseRequested".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnIdleMilliseconds".into(),
                Some(type_registration::default_zero_expr()),
            ),
            (
                "OnIdle".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnExit".into(),
                Some(type_registration::default_none_expr()),
            ),
        ],
    );

    let callbacks = GraphCallbackTypes {
        on_key_pressed,
        on_event,
        on_resize,
        on_paint,
        on_close_requested,
        on_idle,
        on_exit,
    };

    (application_handlers, callbacks)
}
