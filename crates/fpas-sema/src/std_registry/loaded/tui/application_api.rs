use super::super::super::{define_func, define_proc, p};
use super::TuiTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the user-facing `Std.Tui.Application` calls.
///
/// **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).
pub(super) fn register_application_api(checker: &mut Checker, types: &TuiTypes) {
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
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_SHOW_MODAL,
        vec![
            p("App", types.application.clone(), false),
            p("ModalId", Ty::Integer, false),
            p("RootViewId", Ty::Integer, false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SHOW_DIALOG,
        vec![
            p("App", types.application.clone(), false),
            p("ModalId", Ty::Integer, false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE_MODAL,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SIZE,
        vec![p("App", types.application.clone(), false)],
        types.size.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_READ_EVENT,
        vec![p("App", types.application.clone(), false)],
        types.event.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_READ_EVENT_TIMEOUT,
        vec![
            p("App", types.application.clone(), false),
            p("Milliseconds", Ty::Integer, false),
        ],
        Ty::Option(Box::new(types.event.clone())),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_POLL_EVENT,
        vec![p("App", types.application.clone(), false)],
        Ty::Option(Box::new(types.event.clone())),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_REQUEST_REDRAW,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_REDRAW_PENDING,
        vec![p("App", types.application.clone(), false)],
        Ty::Boolean,
    );
}