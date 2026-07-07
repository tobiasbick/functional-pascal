use super::super::super::{define_func, define_proc, p};
use super::TuiTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register try-2 `Std.Tui` symbols (coexists with try-1 on `refactor/tui-try-2`).
///
/// **Documentation:** `docs/refactor-tui-try-2/target-api.md`
pub(super) fn register_try2_api(checker: &mut Checker, types: &TuiTypes) {
    for name in [
        s::STD_TUI_CM_OK,
        s::STD_TUI_CM_CANCEL,
        s::STD_TUI_CM_CLOSE,
        s::STD_TUI_CM_QUIT,
    ] {
        super::super::super::define_const(checker, name, Ty::Integer);
    }

    define_func(
        checker,
        s::STD_TUI_DIALOG_NEW_MODAL,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.dialog.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_BUTTON_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("Command", Ty::Integer, false),
            p("IsDefault", Ty::Boolean, false),
        ],
        types.button.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_DIALOG_ADD,
        vec![
            p("Dlg", types.dialog.clone(), false),
            p("Child", types.button.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_DIALOG_ADD_BUTTON,
        vec![
            p("Dlg", types.dialog.clone(), false),
            p("Bounds", types.rect.clone(), false),
            p("Text", Ty::String, false),
            p("Command", Ty::Integer, false),
            p("IsDefault", Ty::Boolean, false),
        ],
        types.button.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_EXEC_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("Dialog", types.dialog.clone(), false),
        ],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TRY2_INJECT_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("Command", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_TRY2_INJECT_KEYBOARD,
        vec![
            p("App", types.application.clone(), false),
            p("KeyCode", Ty::Integer, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_WINDOW_NEW,
        vec![
            p("Bounds", types.rect.clone(), false),
            p("Title", Ty::String, false),
        ],
        types.window.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_WINDOW_ADD,
        vec![
            p("Win", types.window.clone(), false),
            p("Child", types.button.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_DESKTOP_ADD,
        vec![
            p("App", types.application.clone(), false),
            p("Win", types.window.clone(), false),
        ],
    );
}
