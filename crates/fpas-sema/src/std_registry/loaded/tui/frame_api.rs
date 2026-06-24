//! Frame-root host API registration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use super::super::super::{define_func, p};
use super::TuiTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register frame-root host calls and query state.
pub(super) fn register(checker: &mut Checker, types: &TuiTypes) {
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_DESKTOP_WORK_AREA,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_FRAME_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("Title", Ty::String, false),
            p("Kind", Ty::Integer, false),
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
        s::STD_TUI_APPLICATION_HOST_ACTIVATE_NEXT_WINDOW,
        vec![p("App", types.application.clone(), false)],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_ZOOM_FRAME_ROOT,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
        Ty::Boolean,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_RESTORE_FRAME_ROOT,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
        Ty::Boolean,
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
        s::STD_TUI_APPLICATION_HOST_CASCADE_FRAME_ROOTS,
        vec![
            p("App", types.application.clone(), false),
            p("StepX", Ty::Integer, false),
            p("StepY", Ty::Integer, false),
        ],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_TILE_FRAME_ROOTS,
        vec![p("App", types.application.clone(), false)],
        Ty::Integer,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_FRAME_CONTENT_SIZE,
        vec![
            p("App", types.application.clone(), false),
            p("FrameView", types.view_id.clone(), false),
            p("ContentWidth", Ty::Integer, false),
            p("ContentHeight", Ty::Integer, false),
        ],
        Ty::Unit,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_SCROLL_FRAME,
        vec![
            p("App", types.application.clone(), false),
            p("FrameView", types.view_id.clone(), false),
            p("DeltaX", Ty::Integer, false),
            p("DeltaY", Ty::Integer, false),
        ],
        Ty::Unit,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_FRAME_SCROLL_OFFSET,
        vec![
            p("App", types.application.clone(), false),
            p("FrameView", types.view_id.clone(), false),
            p("OffsetX", Ty::Integer, false),
            p("OffsetY", Ty::Integer, false),
        ],
        Ty::Unit,
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
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_ACTIVATE_FRAME_WINDOW,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
        Ty::Boolean,
    );
}
