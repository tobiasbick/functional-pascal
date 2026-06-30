use super::super::super::p;
use super::TuiCallbackTypes;
use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_std::std_symbols as s;

/// Registered `Std.Tui` types needed to build handler callback signatures.
pub(super) struct TuiRegistrationTypes<'a> {
    /// `Std.Tui.Application` type.
    pub application: &'a Ty,
    /// `Std.Tui.Size` type.
    pub size: &'a Ty,
    /// `Std.Console.KeyEvent` type.
    pub key_event: &'a Ty,
    /// `Std.Console.Event` type.
    pub console_event: &'a Ty,
    /// `Std.Tui.ExitReason` type.
    pub exit_reason: &'a Ty,
}

/// Register callback signatures and the `Std.Tui.ApplicationHandlers` record.
///
/// **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).
pub(super) fn register_application_handlers(
    checker: &mut Checker,
    types: &TuiRegistrationTypes<'_>,
) -> (Ty, TuiCallbackTypes) {
    let &TuiRegistrationTypes {
        application,
        size,
        key_event,
        console_event,
        exit_reason,
    } = types;
    let on_key_pressed = Ty::Function(FunctionTy {
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
    let on_paste = on_mouse.clone();
    let on_focus_gained = on_mouse.clone();
    let on_focus_lost = on_mouse.clone();
    let on_activate = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application.clone(), false)],
        variadic: false,
    });
    let on_deactivate = on_activate.clone();
    let on_command = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
        variadic: false,
    });
    let on_resize = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("NewSize", size.clone(), false),
        ],
        variadic: false,
    });
    let on_paint = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application.clone(), false)],
        variadic: false,
    });
    let on_idle = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application.clone(), false)],
        variadic: false,
    });
    let on_exit = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application.clone(), false),
            p("Reason", exit_reason.clone(), false),
        ],
        variadic: false,
    });
    let application_handlers = type_registration::register_record_type_with_defaults(
        checker,
        s::STD_TUI_APPLICATION_HANDLERS,
        vec![
            ("OnPaint".into(), Ty::Option(Box::new(on_paint.clone()))),
            (
                "OnKeyPressed".into(),
                Ty::Option(Box::new(on_key_pressed.clone())),
            ),
            ("OnMouse".into(), Ty::Option(Box::new(on_mouse.clone()))),
            ("OnPaste".into(), Ty::Option(Box::new(on_paste.clone()))),
            (
                "OnFocusGained".into(),
                Ty::Option(Box::new(on_focus_gained.clone())),
            ),
            (
                "OnFocusLost".into(),
                Ty::Option(Box::new(on_focus_lost.clone())),
            ),
            (
                "OnActivate".into(),
                Ty::Option(Box::new(on_activate.clone())),
            ),
            (
                "OnDeactivate".into(),
                Ty::Option(Box::new(on_deactivate.clone())),
            ),
            ("OnCommand".into(), Ty::Option(Box::new(on_command.clone()))),
            ("OnResize".into(), Ty::Option(Box::new(on_resize.clone()))),
            ("OnIdleMilliseconds".into(), Ty::Integer),
            ("OnIdle".into(), Ty::Option(Box::new(on_idle.clone()))),
            ("OnExit".into(), Ty::Option(Box::new(on_exit.clone()))),
        ],
        vec![
            (
                "OnPaint".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnKeyPressed".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnMouse".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnPaste".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnFocusGained".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnFocusLost".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnActivate".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnDeactivate".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnCommand".into(),
                Some(type_registration::default_none_expr()),
            ),
            (
                "OnResize".into(),
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

    let callbacks = TuiCallbackTypes { on_command };

    (application_handlers, callbacks)
}
