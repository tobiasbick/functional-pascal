//! `Std.Tui` semantic registration.
//!
//! `Std.Tui.TuiEvent.key` uses `Std.Console.KeyEvent` (registered by
//! [`super::console::register_std_console_key_api`] when needed).
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

mod application_api;
mod handlers;
mod host_api;

use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::std_symbols as s;
use fpas_std::{TUI_EVENT_KIND_VARIANTS, TUI_EXIT_REASON_VARIANTS};

struct TuiTypes {
    application: Ty,
    size: Ty,
    screen_cell: Ty,
    key_event: Ty,
    console_event: Ty,
    application_handlers: Ty,
    menu_bar_item: Ty,
    menu_bar_style: Ty,
    status_bar_segment: Ty,
    status_bar_style: Ty,
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
    let screen_cell = type_registration::register_record_type(
        checker,
        s::STD_TUI_SCREEN_CELL,
        vec![
            ("ch".into(), Ty::Char),
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
    type_registration::register_record_type_with_defaults(
        checker,
        s::STD_TUI_MENU_POPUP_ITEM,
        vec![
            ("Label".into(), Ty::String),
            ("Shortcut".into(), Ty::Char),
            ("Enabled".into(), Ty::Boolean),
            ("CommandId".into(), Ty::Integer),
            ("Separator".into(), Ty::Boolean),
        ],
        vec![
            ("Label".into(), None),
            ("Shortcut".into(), None),
            ("Enabled".into(), None),
            ("CommandId".into(), None),
            (
                "Separator".into(),
                Some(type_registration::default_false_expr()),
            ),
        ],
    );
    type_registration::register_record_type(
        checker,
        s::STD_TUI_MENU_BAR_ITEM,
        vec![
            ("Label".into(), Ty::String),
            ("Shortcut".into(), Ty::Char),
            ("Enabled".into(), Ty::Boolean),
            ("CommandId".into(), Ty::Integer),
            (
                "Submenu".into(),
                Ty::Array(Box::new(type_registration::lookup_required_type(
                    checker,
                    s::STD_TUI_MENU_POPUP_ITEM,
                    "MenuPopupItem",
                ))),
            ),
        ],
    );
    let menu_bar_item =
        type_registration::lookup_required_type(checker, s::STD_TUI_MENU_BAR_ITEM, "MenuBarItem");
    type_registration::register_record_type(
        checker,
        s::STD_TUI_MENU_BAR_STYLE,
        vec![
            ("BarBg".into(), Ty::Integer),
            ("BarFg".into(), Ty::Integer),
            ("AccelFg".into(), Ty::Integer),
            ("HighlightBg".into(), Ty::Integer),
            ("HighlightFg".into(), Ty::Integer),
            ("DisabledFg".into(), Ty::Integer),
        ],
    );
    let menu_bar_style =
        type_registration::lookup_required_type(checker, s::STD_TUI_MENU_BAR_STYLE, "MenuBarStyle");
    type_registration::register_record_type(
        checker,
        s::STD_TUI_STATUS_BAR_SEGMENT,
        vec![
            ("Text".into(), Ty::String),
            ("AlignRight".into(), Ty::Boolean),
        ],
    );
    let status_bar_segment = type_registration::lookup_required_type(
        checker,
        s::STD_TUI_STATUS_BAR_SEGMENT,
        "StatusBarSegment",
    );
    type_registration::register_record_type(
        checker,
        s::STD_TUI_STATUS_BAR_STYLE,
        vec![("BarBg".into(), Ty::Integer), ("BarFg".into(), Ty::Integer)],
    );
    let status_bar_style = type_registration::lookup_required_type(
        checker,
        s::STD_TUI_STATUS_BAR_STYLE,
        "StatusBarStyle",
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
    type_registration::register_record_type(
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
        screen_cell,
        key_event,
        console_event,
        application_handlers,
        menu_bar_item,
        menu_bar_style,
        status_bar_segment,
        status_bar_style,
    };
    application_api::register_application_api(checker, &types);
    host_api::register_host_api(checker, &types, &callbacks);
}
