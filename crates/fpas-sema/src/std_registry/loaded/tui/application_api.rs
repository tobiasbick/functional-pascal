use super::super::super::{define_func, define_proc, p};
use super::{TuiCallbackTypes, TuiTypes};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register lifecycle, chrome, and test helpers on `Std.Tui.Application`.
///
/// View construction lives in try-2 `Dialog.NewModal`, `Button.New`, `Window.New`, etc.
///
/// **Documentation:** `docs/pascal/std/tui/session.md`, `docs/refactor-tui-try-2/target-api.md`
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
    define_func(
        checker,
        s::STD_TUI_APPLICATION_NEW,
        vec![],
        types.application.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE,
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
        s::STD_TUI_APPLICATION_SET_MENU_BAR,
        vec![
            p("App", types.application.clone(), false),
            p("MenuBar", types.menu_bar.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_SET_STATUS_LINE,
        vec![
            p("App", types.application.clone(), false),
            p("StatusLine", types.status_line.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_QUIT,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ON_KEY,
        vec![
            p("App", types.application.clone(), false),
            p("OnKey", callbacks.on_key.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_ON_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("OnMouse", callbacks.on_mouse.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_CLICK_BUTTON,
        vec![
            p("App", types.application.clone(), false),
            p("Button", types.button.clone(), false),
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
        s::STD_TUI_APPLICATION_TEST_DISPATCH_MENU_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("MenuBar", types.menu_bar.clone(), false),
            p("MenuIndex", Ty::Integer, false),
            p("ItemIndex", Ty::Integer, false),
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
        s::STD_TUI_APPLICATION_CLOSE_FOR_TEST,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT,
        vec![
            p("App", types.application.clone(), false),
            p("Result", Ty::Option(Box::new(Ty::String)), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT,
        vec![
            p("App", types.application.clone(), false),
            p("Command", Ty::Integer, false),
        ],
    );
}
