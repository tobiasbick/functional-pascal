//! Frame-root retained dialog and query registration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use super::super::super::{define_func, p};
use super::TuiTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register frame-root calls and query state that remain public during the Turbo Vision rewrite.
pub(super) fn register(checker: &mut Checker, types: &TuiTypes) {
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SHOW_FRAMED_DIALOG,
        vec![
            p("App", types.application.clone(), false),
            p("ModalId", Ty::Integer, false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("Title", Ty::String, false),
            p("Movable", Ty::Boolean, false),
            p("Resizable", Ty::Boolean, false),
            p("Zoomable", Ty::Boolean, false),
            p("Scrollable", Ty::Boolean, false),
            p("Closable", Ty::Boolean, false),
        ],
        types.view_id.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_QUERY_FRAME_ROOT_STATE,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
        types.controls.frame_root_state.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_QUERY_FRAME_SCROLL_STATE,
        vec![
            p("App", types.application.clone(), false),
            p("FrameView", types.view_id.clone(), false),
        ],
        types.controls.frame_scroll_state.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_QUERY_FRAME_WINDOW_LIST,
        vec![p("App", types.application.clone(), false)],
        Ty::Array(Box::new(types.controls.frame_window_entry.clone())),
    );
}
