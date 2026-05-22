//! `Std.Tui` semantic registration.
//!
//! `Std.Tui.TuiEvent.key` uses `Std.Console.KeyEvent` (registered by
//! [`super::console::register_std_console_key_api`] when needed).
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

mod application_api;
mod handlers;
mod host_api;
mod type_registration;

use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;
use fpas_std::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS};

struct TuiTypes {
    application: Ty,
    size: Ty,
    key_event: Ty,
    event: Ty,
    application_handlers: Ty,
}

struct TuiCallbackTypes {
    on_key_pressed: Ty,
    on_mouse: Ty,
    on_paste: Ty,
    on_focus_gained: Ty,
    on_focus_lost: Ty,
    on_activate: Ty,
    on_deactivate: Ty,
    on_command: Ty,
    on_view_paint: Ty,
    on_resize: Ty,
    on_paint: Ty,
    on_idle: Ty,
    on_exit: Ty,
}

/// Register the `Std.Tui` semantic surface, including the application API and host bridge.
///
/// **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).
pub(super) fn register_std_tui(checker: &mut Checker) {
    let application =
        type_registration::register_record_type(checker, s::STD_TUI_APPLICATION, Vec::new());
    let rect = type_registration::register_record_type(
        checker,
        s::STD_TUI_RECT,
        vec![
            ("x".into(), Ty::Integer),
            ("y".into(), Ty::Integer),
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );
    let size = type_registration::register_record_type(
        checker,
        s::STD_TUI_SIZE,
        vec![
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );
    let key_event = lookup_required_type(
        checker,
        s::STD_CONSOLE_KEY_EVENT,
        "Std.Console.KeyEvent must be registered before Std.Tui (see loaded/mod.rs)",
    );
    let console_event = lookup_required_type(
        checker,
        s::STD_CONSOLE_EVENT,
        "Std.Console.Event must be registered before Std.Tui (see loaded/mod.rs)",
    );
    let event_kind = type_registration::register_enum_type(
        checker,
        s::STD_TUI_EVENT_KIND,
        TUI_EVENT_KIND_VARIANTS,
    );
    let exit_reason = type_registration::register_enum_type(
        checker,
        s::STD_TUI_EXIT_REASON,
        TUI_EXIT_REASON_VARIANTS,
    );
    let (application_handlers, callbacks) = handlers::register_application_handlers(
        checker,
        &application,
        &rect,
        &size,
        &key_event,
        &console_event,
        &exit_reason,
    );
    let event = type_registration::register_record_type(
        checker,
        s::STD_TUI_EVENT,
        vec![
            ("kind".into(), event_kind),
            ("key".into(), key_event.clone()),
            ("size".into(), size.clone()),
        ],
    );

    let types = TuiTypes {
        application,
        size,
        key_event,
        event,
        application_handlers,
    };
    application_api::register_application_api(checker, &types);
    host_api::register_host_api(checker, &types, &callbacks);
}

fn lookup_required_type(checker: &Checker, qualified_name: &str, message: &str) -> Ty {
    checker
        .scopes
        .lookup(qualified_name)
        .map(|symbol| symbol.ty.clone())
        .unwrap_or_else(|| unreachable!("{message}"))
}
