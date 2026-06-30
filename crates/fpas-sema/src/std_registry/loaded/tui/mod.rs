//! `Std.Tui` semantic registration.
//!
//! `Std.Tui.TuiEvent.key` uses `Std.Console.KeyEvent` (registered by
//! [`super::console::register_std_console_key_api`] when needed).
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod application_api;
mod command_api;
mod handlers;

use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::std_symbols as s;
use fpas_std::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS};

struct TuiTypes {
    application: Ty,
    dialog: Ty,
    window: Ty,
    button: Ty,
    rect: Ty,
    size: Ty,
    screen_cell: Ty,
    key_event: Ty,
    console_event: Ty,
    application_handlers: Ty,
}

struct TuiCallbackTypes {
    on_command: Ty,
}

/// Register the current `Std.Tui` semantic surface.
///
/// **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).
pub(super) fn register_std_tui(checker: &mut Checker) {
    let application =
        type_registration::register_record_type(checker, s::STD_TUI_APPLICATION, Vec::new());
    type_registration::register_record_type(checker, s::STD_TUI_VIEW_ID, Vec::new());
    let dialog = type_registration::register_record_type(checker, s::STD_TUI_DIALOG, Vec::new());
    let window = type_registration::register_record_type(checker, s::STD_TUI_WINDOW, Vec::new());
    let button = type_registration::register_record_type(checker, s::STD_TUI_BUTTON, Vec::new());
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
    type_registration::register_record_type(
        checker,
        s::STD_TUI_POINT,
        vec![("x".into(), Ty::Integer), ("y".into(), Ty::Integer)],
    );
    let size = type_registration::register_record_type(
        checker,
        s::STD_TUI_SIZE,
        vec![
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );
    let screen_cell = type_registration::register_record_type(
        checker,
        s::STD_TUI_SCREEN_CELL,
        vec![
            ("ch".into(), Ty::String),
            ("fg".into(), Ty::Integer),
            ("bg".into(), Ty::Integer),
        ],
    );
    let key_event = type_registration::lookup_required_type(
        checker,
        s::STD_CONSOLE_KEY_EVENT,
        "Std.Console.KeyEvent must be registered before Std.Tui (see loaded/mod.rs)",
    );
    let console_event = type_registration::lookup_required_type(
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
        &handlers::TuiRegistrationTypes {
            application: &application,
            size: &size,
            key_event: &key_event,
            console_event: &console_event,
            exit_reason: &exit_reason,
        },
    );
    type_registration::register_record_type(
        checker,
        s::STD_TUI_EVENT,
        vec![
            ("kind".into(), event_kind),
            ("key".into(), key_event.clone()),
            ("size".into(), size.clone()),
        ],
    );

    command_api::register_command_constants(checker);
    super::super::builtins::register_add_child_builtin(checker);

    let types = TuiTypes {
        application,
        dialog,
        window,
        button,
        rect,
        size,
        screen_cell,
        key_event,
        console_event,
        application_handlers,
    };
    application_api::register_application_api(checker, &types, &callbacks);
}
