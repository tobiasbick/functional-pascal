use super::super::super::{define_func, define_proc, p};
use super::{TuiCallbackTypes, TuiTypes};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the host-dispatch bridge for `Std.Tui.Application`.
///
/// **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).
pub(super) fn register_host_api(
    checker: &mut Checker,
    types: &TuiTypes,
    callbacks: &TuiCallbackTypes,
) {
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
        vec![
            p("App", types.application.clone(), false),
            p("OnKeyPressed", callbacks.on_key_pressed.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED,
        vec![
            p("App", types.application.clone(), false),
            p("Key", types.key_event.clone(), false),
        ],
        Ty::Boolean,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE,
        vec![
            p("App", types.application.clone(), false),
            p("OnResize", callbacks.on_resize.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_PROCESS_NEXT,
        vec![
            p("App", types.application.clone(), false),
            p("MaxSpins", Ty::Integer, false),
        ],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PAINT,
        vec![
            p("App", types.application.clone(), false),
            p("OnPaint", callbacks.on_paint.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE,
        vec![
            p("App", types.application.clone(), false),
            p("Milliseconds", Ty::Integer, false),
            p("OnIdle", callbacks.on_idle.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW,
        vec![p("App", types.application.clone(), false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_RUN_LOOP,
        vec![
            p("App", types.application.clone(), false),
            p("MaxIterations", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REQUEST_QUIT,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_EXIT,
        vec![
            p("App", types.application.clone(), false),
            p("OnExit", callbacks.on_exit.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("OnMouse", callbacks.on_mouse.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE,
        vec![
            p("App", types.application.clone(), false),
            p("OnPaste", callbacks.on_paste.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED,
        vec![
            p("App", types.application.clone(), false),
            p("OnFocusGained", callbacks.on_focus_gained.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST,
        vec![
            p("App", types.application.clone(), false),
            p("OnFocusLost", callbacks.on_focus_lost.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_ACTIVATE,
        vec![
            p("App", types.application.clone(), false),
            p("OnActivate", callbacks.on_activate.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_DEACTIVATE,
        vec![
            p("App", types.application.clone(), false),
            p("OnDeactivate", callbacks.on_deactivate.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("OnCommand", callbacks.on_command.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_BIND_COMMAND,
        vec![
            p("App", types.application.clone(), false),
            p("Key", types.key_event.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("Key", types.key_event.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_ACTIVE_MODAL,
        vec![
            p("App", types.application.clone(), false),
            p("Key", types.key_event.clone(), false),
            p("CommandId", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_ENTER_MODAL,
        vec![
            p("App", types.application.clone(), false),
            p("ModalId", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_LEAVE_MODAL,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
        types.view_id.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_UNREGISTER_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_PUSH_CHILD_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_ATTACH_VIEW_TO_ACTIVE_MODAL,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_VIEW_RECT,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_VIEW_PARENT,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("Parent", Ty::Option(Box::new(types.view_id.clone())), false),
        ],
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_REGISTER_ON_VIEW_PAINT,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p("OnViewPaint", callbacks.on_view_paint.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_SOLID_FILL_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("FillColor", Ty::Integer, false),
            p("TextColor", Ty::Option(Box::new(Ty::Integer)), false),
            p("FillChar", Ty::Option(Box::new(Ty::Char)), false),
        ],
        types.view_id.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_MENU_BAR_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p(
                "Items",
                Ty::Array(Box::new(types.menu_bar_item.clone())),
                false,
            ),
            p("Style", types.menu_bar_style.clone(), false),
        ],
        types.view_id.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_MENU_BAR_ITEMS,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p(
                "Items",
                Ty::Array(Box::new(types.menu_bar_item.clone())),
                false,
            ),
        ],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_HOST_CREATE_STATUS_BAR_VIEW,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p(
                "Segments",
                Ty::Array(Box::new(types.status_bar_segment.clone())),
                false,
            ),
            p("Style", types.status_bar_style.clone(), false),
        ],
        types.view_id.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_HOST_SET_STATUS_BAR_SEGMENTS,
        vec![
            p("App", types.application.clone(), false),
            p("ViewId", types.view_id.clone(), false),
            p(
                "Segments",
                Ty::Array(Box::new(types.status_bar_segment.clone())),
                false,
            ),
        ],
    );
}
