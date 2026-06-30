use super::super::super::{define_func, define_proc, p};
use super::{TuiCallbackTypes, TuiTypes};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the user-facing `Std.Tui.Application` calls.
///
/// **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).
pub(super) fn register_application_api(
    checker: &mut Checker,
    types: &TuiTypes,
    callbacks: &TuiCallbackTypes,
) {
    define_func(
        checker,
        s::STD_TUI_APPLICATION_OPEN,
        vec![],
        types.application.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CONFIGURE,
        vec![
            p("App", types.application.clone(), false),
            p("Handlers", types.application_handlers.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_RUN,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SIZE,
        vec![p("App", types.application.clone(), false)],
        types.size.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_REQUEST_REDRAW,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_DIALOG,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.dialog.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_CREATE_BUTTON,
        vec![
            p("App", types.application.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("CommandId", Ty::Integer, false),
        ],
        types.button.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ADD_CHILD,
        vec![
            p("App", types.application.clone(), false),
            p("Dialog", types.dialog.clone(), false),
            p("Child", types.button.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ON_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("OnCommand", callbacks.on_command.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_PUMP,
        vec![p("App", types.application.clone(), false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_QUIT,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_CLICK_BUTTON,
        vec![
            p("App", types.application.clone(), false),
            p("Button", types.button.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_OPEN_FOR_TEST,
        vec![
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
        types.application.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_PUMP,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_PUMP_UNTIL_IDLE,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE_FOR_TEST,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_SEND_KEY,
        vec![
            p("App", types.application.clone(), false),
            p("Key", types.key_event.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_SEND_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("Event", types.console_event.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_MOVE_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_CLICK_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_RESIZE,
        vec![
            p("App", types.application.clone(), false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_PASTE,
        vec![
            p("App", types.application.clone(), false),
            p("Text", Ty::String, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_FOCUS,
        vec![
            p("App", types.application.clone(), false),
            p("Gained", Ty::Boolean, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_QUERY_SCREEN_SIZE,
        vec![p("App", types.application.clone(), false)],
        types.size.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_QUERY_SCREEN_LINE,
        vec![
            p("App", types.application.clone(), false),
            p("Y", Ty::Integer, false),
        ],
        Ty::String,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_QUERY_SCREEN_CELL,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
        ],
        types.screen_cell.clone(),
    );
}
