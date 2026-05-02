//! `Std.Tui` semantic registration.
//!
//! `Std.Tui.TuiEvent.key` uses `Std.Console.KeyEvent` (registered by [`super::console::register_std_console_key_api`] when needed).
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use super::super::{define_func, define_proc, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, FunctionTy, ProcedureTy, RecordTy, Ty};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;
use fpas_std::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS};

fn register_enum_type(checker: &mut Checker, qualified_name: &str, variants: &[&str]) -> Ty {
    let variants: Vec<EnumVariantTy> = variants
        .iter()
        .map(|variant| EnumVariantTy {
            name: (*variant).to_string(),
            fields: vec![],
        })
        .collect();
    let member_names: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
    let enum_ty = Ty::Enum(EnumTy {
        name: qualified_name.into(),
        variants,
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: enum_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );

    for member in &member_names {
        let qualified = format!("{qualified_name}.{member}");
        checker.scopes.define(
            &qualified,
            Symbol {
                ty: enum_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumMember,
            },
        );
    }

    enum_ty
}

fn register_record_type(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
) -> Ty {
    let record_ty = Ty::Record(RecordTy {
        name: qualified_name.into(),
        fields,
        methods: Vec::new(),
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: record_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );
    record_ty
}

fn register_record_type_with_defaults(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
    defaults: Vec<(String, Option<Expr>)>,
) -> Ty {
    let record_ty = register_record_type(checker, qualified_name, fields);
    if defaults.iter().any(|(_, default)| default.is_some()) {
        checker
            .record_defaults
            .insert(qualified_name.to_string(), defaults);
    }
    record_ty
}

fn builtin_span() -> Span {
    Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
        source_id: 0,
    }
}

fn default_none_expr() -> Expr {
    Expr::OptionNone(builtin_span())
}

fn default_zero_expr() -> Expr {
    Expr::Integer(0, builtin_span())
}

pub(super) fn register_std_tui(checker: &mut Checker) {
    let application_ty = register_record_type(checker, s::STD_TUI_APPLICATION, Vec::new());
    let size_ty = register_record_type(
        checker,
        s::STD_TUI_SIZE,
        vec![
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );

    let key_event_ty = match checker.scopes.lookup(s::STD_CONSOLE_KEY_EVENT) {
        Some(sym) => sym.ty.clone(),
        None => unreachable!(
            "Std.Console.KeyEvent must be registered before Std.Tui (see loaded/mod.rs)"
        ),
    };

    let console_event_ty = match checker.scopes.lookup(s::STD_CONSOLE_EVENT) {
        Some(sym) => sym.ty.clone(),
        None => {
            unreachable!("Std.Console.Event must be registered before Std.Tui (see loaded/mod.rs)")
        }
    };

    let event_kind_ty = register_enum_type(checker, s::STD_TUI_EVENT_KIND, TUI_EVENT_KIND_VARIANTS);
    let exit_reason_ty =
        register_enum_type(checker, s::STD_TUI_EXIT_REASON, TUI_EXIT_REASON_VARIANTS);
    let on_key_pressed_ty = Ty::Function(FunctionTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("Key", key_event_ty.clone(), false),
        ],
        return_type: Box::new(Ty::Boolean),
        variadic: false,
    });
    let on_mouse_ty = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("Event", console_event_ty.clone(), false),
        ],
        variadic: false,
    });
    // OnPaste, OnFocusGained, OnFocusLost all share the same signature as OnMouse.
    let on_paste_ty = on_mouse_ty.clone();
    let on_focus_gained_ty = on_mouse_ty.clone();
    let on_focus_lost_ty = on_mouse_ty.clone();
    // OnActivate and OnDeactivate fire on host-managed focus transitions (Tab / Shift+Tab).
    let on_activate_ty = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application_ty.clone(), false)],
        variadic: false,
    });
    let on_deactivate_ty = on_activate_ty.clone();
    let on_resize_ty = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("NewSize", size_ty.clone(), false),
        ],
        variadic: false,
    });
    let on_paint_ty = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application_ty.clone(), false)],
        variadic: false,
    });
    let on_idle_ty = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![p("App", application_ty.clone(), false)],
        variadic: false,
    });
    let on_exit_ty = Ty::Procedure(ProcedureTy {
        type_params: Vec::new(),
        params: vec![
            p("App", application_ty.clone(), false),
            p("Reason", exit_reason_ty.clone(), false),
        ],
        variadic: false,
    });
    let application_handlers_ty = register_record_type_with_defaults(
        checker,
        s::STD_TUI_APPLICATION_HANDLERS,
        vec![
            ("OnPaint".into(), on_paint_ty.clone()),
            (
                "OnKeyPressed".into(),
                Ty::Option(Box::new(on_key_pressed_ty.clone())),
            ),
            ("OnMouse".into(), Ty::Option(Box::new(on_mouse_ty.clone()))),
            ("OnPaste".into(), Ty::Option(Box::new(on_paste_ty.clone()))),
            (
                "OnFocusGained".into(),
                Ty::Option(Box::new(on_focus_gained_ty.clone())),
            ),
            (
                "OnFocusLost".into(),
                Ty::Option(Box::new(on_focus_lost_ty.clone())),
            ),
            (
                "OnActivate".into(),
                Ty::Option(Box::new(on_activate_ty.clone())),
            ),
            (
                "OnDeactivate".into(),
                Ty::Option(Box::new(on_deactivate_ty.clone())),
            ),
            (
                "OnResize".into(),
                Ty::Option(Box::new(on_resize_ty.clone())),
            ),
            ("OnIdleMilliseconds".into(), Ty::Integer),
            ("OnIdle".into(), Ty::Option(Box::new(on_idle_ty.clone()))),
            ("OnExit".into(), Ty::Option(Box::new(on_exit_ty.clone()))),
        ],
        vec![
            ("OnPaint".into(), None),
            ("OnKeyPressed".into(), Some(default_none_expr())),
            ("OnMouse".into(), Some(default_none_expr())),
            ("OnPaste".into(), Some(default_none_expr())),
            ("OnFocusGained".into(), Some(default_none_expr())),
            ("OnFocusLost".into(), Some(default_none_expr())),
            ("OnActivate".into(), Some(default_none_expr())),
            ("OnDeactivate".into(), Some(default_none_expr())),
            ("OnResize".into(), Some(default_none_expr())),
            ("OnIdleMilliseconds".into(), Some(default_zero_expr())),
            ("OnIdle".into(), Some(default_none_expr())),
            ("OnExit".into(), Some(default_none_expr())),
        ],
    );
    let event_ty = register_record_type(
        checker,
        s::STD_TUI_EVENT,
        vec![
            ("kind".into(), event_kind_ty),
            ("key".into(), key_event_ty.clone()),
            ("size".into(), size_ty.clone()),
        ],
    );

    define_func(
        checker,
        s::STD_TUI_APPLICATION_OPEN,
        vec![],
        application_ty.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE,
        vec![p("App", application_ty.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CONFIGURE,
        vec![
            p("App", application_ty.clone(), false),
            p("Handlers", application_handlers_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_RUN,
        vec![p("App", application_ty.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SIZE,
        vec![p("App", application_ty.clone(), false)],
        size_ty.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_READ_EVENT,
        vec![p("App", application_ty.clone(), false)],
        event_ty.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_READ_EVENT_TIMEOUT,
        vec![
            p("App", application_ty.clone(), false),
            p("Milliseconds", Ty::Integer, false),
        ],
        Ty::Option(Box::new(event_ty.clone())),
    );

    // Host dispatch bridge (bytecode intrinsics 255–263); see `docs/pascal/std/tui-app.md`.
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_POLL_NEXT,
        vec![p("App", application_ty.clone(), false)],
        Ty::Option(Box::new(event_ty.clone())),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
        vec![
            p("App", application_ty.clone(), false),
            p("OnKeyPressed", on_key_pressed_ty, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED,
        vec![
            p("App", application_ty.clone(), false),
            p("Key", key_event_ty.clone(), false),
        ],
        Ty::Boolean,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE,
        vec![
            p("App", application_ty.clone(), false),
            p("OnResize", on_resize_ty, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_PROCESS_NEXT,
        vec![
            p("App", application_ty.clone(), false),
            p("MaxSpins", Ty::Integer, false),
        ],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PAINT,
        vec![
            p("App", application_ty.clone(), false),
            p("OnPaint", on_paint_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE,
        vec![
            p("App", application_ty.clone(), false),
            p("Milliseconds", Ty::Integer, false),
            p("OnIdle", on_idle_ty, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW,
        vec![p("App", application_ty.clone(), false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_RUN_LOOP,
        vec![
            p("App", application_ty.clone(), false),
            p("MaxIterations", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REQUEST_QUIT,
        vec![p("App", application_ty.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_EXIT,
        vec![
            p("App", application_ty.clone(), false),
            p("OnExit", on_exit_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE,
        vec![
            p("App", application_ty.clone(), false),
            p("OnMouse", on_mouse_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE,
        vec![
            p("App", application_ty.clone(), false),
            p("OnPaste", on_paste_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED,
        vec![
            p("App", application_ty.clone(), false),
            p("OnFocusGained", on_focus_gained_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST,
        vec![
            p("App", application_ty.clone(), false),
            p("OnFocusLost", on_focus_lost_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_ACTIVATE,
        vec![
            p("App", application_ty.clone(), false),
            p("OnActivate", on_activate_ty, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_DEACTIVATE,
        vec![
            p("App", application_ty.clone(), false),
            p("OnDeactivate", on_deactivate_ty, false),
        ],
    );

    define_func(
        checker,
        s::STD_TUI_APPLICATION_POLL_EVENT,
        vec![p("App", application_ty.clone(), false)],
        Ty::Option(Box::new(event_ty)),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_REQUEST_REDRAW,
        vec![p("App", application_ty.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_REDRAW_PENDING,
        vec![p("App", application_ty, false)],
        Ty::Boolean,
    );
}
