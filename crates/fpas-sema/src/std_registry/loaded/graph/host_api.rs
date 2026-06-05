use super::super::super::{define_func, define_proc, p};
use super::{GraphCallbackTypes, GraphTypes};
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the host-dispatch bridge for `Std.Graph.Application`.
///
/// **Documentation:** `docs/pascal/std/graph-app.md` (from the repository root).
pub(super) fn register_host_api(
    checker: &mut Checker,
    types: &GraphTypes,
    callbacks: &GraphCallbackTypes,
) {
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
        vec![
            p("App", types.application.clone(), false),
            p("OnKeyPressed", callbacks.on_key_pressed.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_RESIZE,
        vec![
            p("App", types.application.clone(), false),
            p("OnResize", callbacks.on_resize.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_PROCESS_NEXT,
        vec![
            p("App", types.application.clone(), false),
            p("MaxSpins", Ty::Integer, false),
        ],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_PAINT,
        vec![
            p("App", types.application.clone(), false),
            p("OnPaint", callbacks.on_paint.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_IDLE,
        vec![
            p("App", types.application.clone(), false),
            p("Milliseconds", Ty::Integer, false),
            p("OnIdle", callbacks.on_idle.clone(), false),
        ],
    );
    define_func(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_DISPATCH_REDRAW,
        vec![p("App", types.application.clone(), false)],
        Ty::Integer,
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REQUEST_QUIT,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_EXIT,
        vec![
            p("App", types.application.clone(), false),
            p("OnExit", callbacks.on_exit.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_MOUSE,
        vec![
            p("App", types.application.clone(), false),
            p("OnMouse", callbacks.on_event.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_WHEEL,
        vec![
            p("App", types.application.clone(), false),
            p("OnWheel", callbacks.on_event.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_CLOSE_REQUESTED,
        vec![
            p("App", types.application.clone(), false),
            p(
                "OnCloseRequested",
                callbacks.on_close_requested.clone(),
                false,
            ),
        ],
    );
}
