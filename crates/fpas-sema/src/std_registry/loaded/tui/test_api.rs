use super::super::super::{define_proc, p};
use super::TuiTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register headless test helpers under the `Std.Tui.Test.*` namespace.
///
/// **Documentation:** `docs/pascal/std/tui/app/testing.md`
pub(super) fn register_test_api(checker: &mut Checker, types: &TuiTypes) {
    define_proc(
        checker,
        s::STD_TUI_TEST_CLICK,
        vec![
            p("App", types.application.clone(), false),
            p("Button", types.button.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_TEST_DISPATCH_MENU,
        vec![
            p("App", types.application.clone(), false),
            p("MenuBar", types.menu_bar.clone(), false),
            p("MenuIndex", Ty::Integer, false),
            p("ItemIndex", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_TEST_INJECT_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("Command", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_TEST_INJECT_KEYBOARD,
        vec![
            p("App", types.application.clone(), false),
            p("KeyCode", Ty::Integer, false),
        ],
    );
}
