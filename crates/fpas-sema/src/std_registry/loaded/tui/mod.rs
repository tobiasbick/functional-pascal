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
    dialog_result: Ty,
    window: Ty,
    button: Ty,
    static_text: Ty,
    memo: Ty,
    text_viewer: Ty,
    input_line: Ty,
    list_box: Ty,
    check_box: Ty,
    radio_button: Ty,
    menu_bar: Ty,
    menu: Ty,
    status_line: Ty,
    status_item: Ty,
    rect: Ty,
    size: Ty,
    screen_cell: Ty,
    key_event: Ty,
    console_event: Ty,
    application_handlers: Ty,
}

struct TuiCallbackTypes {
    on_command: Ty,
    on_key: Ty,
    on_mouse: Ty,
}

/// Register the current `Std.Tui` semantic surface.
///
/// **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).
pub(super) fn register_std_tui(checker: &mut Checker) {
    let application =
        type_registration::register_record_type(checker, s::STD_TUI_APPLICATION, Vec::new());
    type_registration::register_record_type(checker, s::STD_TUI_VIEW_ID, Vec::new());
    let dialog = type_registration::register_record_type(checker, s::STD_TUI_DIALOG, Vec::new());
    let dialog_result = type_registration::register_record_type(
        checker,
        s::STD_TUI_DIALOG_RESULT,
        vec![("command".into(), Ty::Integer)],
    );
    let window = type_registration::register_record_type(checker, s::STD_TUI_WINDOW, Vec::new());
    let button = type_registration::register_record_type(checker, s::STD_TUI_BUTTON, Vec::new());
    let static_text =
        type_registration::register_record_type(checker, s::STD_TUI_STATIC_TEXT, Vec::new());
    let memo = type_registration::register_record_type(checker, s::STD_TUI_MEMO, Vec::new());
    let text_viewer =
        type_registration::register_record_type(checker, s::STD_TUI_TEXT_VIEWER, Vec::new());
    let input_line =
        type_registration::register_record_type(checker, s::STD_TUI_INPUT_LINE, Vec::new());
    let list_box =
        type_registration::register_record_type(checker, s::STD_TUI_LIST_BOX, Vec::new());
    let check_box =
        type_registration::register_record_type(checker, s::STD_TUI_CHECK_BOX, Vec::new());
    let radio_button =
        type_registration::register_record_type(checker, s::STD_TUI_RADIO_BUTTON, Vec::new());
    let menu_bar =
        type_registration::register_record_type(checker, s::STD_TUI_MENU_BAR, Vec::new());
    let menu_item = type_registration::register_record_type(
        checker,
        s::STD_TUI_MENU_ITEM,
        vec![
            ("text".into(), Ty::String),
            ("commandId".into(), Ty::Integer),
        ],
    );
    let menu = type_registration::register_record_type(
        checker,
        s::STD_TUI_MENU,
        vec![
            ("title".into(), Ty::String),
            ("items".into(), Ty::Array(Box::new(menu_item.clone()))),
        ],
    );
    let status_line =
        type_registration::register_record_type(checker, s::STD_TUI_STATUS_LINE, Vec::new());
    let status_item = type_registration::register_record_type(
        checker,
        s::STD_TUI_STATUS_ITEM,
        vec![
            ("text".into(), Ty::String),
            ("keyCode".into(), Ty::Integer),
            ("commandId".into(), Ty::Integer),
        ],
    );
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
        dialog_result,
        window,
        button,
        static_text,
        memo,
        text_viewer,
        input_line,
        list_box,
        check_box,
        radio_button,
        menu_bar,
        menu,
        status_line,
        status_item,
        rect,
        size,
        screen_cell,
        key_event,
        console_event,
        application_handlers,
    };
    application_api::register_application_api(checker, &types, &callbacks);
}
