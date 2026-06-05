use super::super::super::{define_func, define_proc, p};
use super::GraphTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the user-facing `Std.Graph.Application` calls.
///
/// **Documentation:** `docs/pascal/std/graph.md` (from the repository root).
pub(super) fn register_application_api(checker: &mut Checker, types: &GraphTypes) {
    define_func(
        checker,
        s::STD_GRAPH_APPLICATION_OPEN,
        vec![
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("Title", Ty::String, false),
        ],
        types.application.clone(),
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_CLOSE,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_CONFIGURE,
        vec![
            p("App", types.application.clone(), false),
            p("Handlers", types.application_handlers.clone(), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_RUN,
        vec![p("App", types.application.clone(), false)],
    );
    define_func(
        checker,
        s::STD_GRAPH_APPLICATION_SIZE,
        vec![p("App", types.application.clone(), false)],
        types.size.clone(),
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_REQUEST_REDRAW,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_UPLOAD_FRAME,
        vec![
            p("App", types.application.clone(), false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("Pixels", Ty::Array(Box::new(Ty::Integer)), false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_CLEAR,
        vec![
            p("App", types.application.clone(), false),
            p("Color", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_PUT_PIXEL,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Color", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_PRESENT,
        vec![p("App", types.application.clone(), false)],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_DRAW_LINE,
        vec![
            p("App", types.application.clone(), false),
            p("X1", Ty::Integer, false),
            p("Y1", Ty::Integer, false),
            p("X2", Ty::Integer, false),
            p("Y2", Ty::Integer, false),
            p("Color", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_DRAW_RECT,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("Color", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_FILL_RECT,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Width", Ty::Integer, false),
            p("Height", Ty::Integer, false),
            p("Color", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_DRAW_CIRCLE,
        vec![
            p("App", types.application.clone(), false),
            p("CenterX", Ty::Integer, false),
            p("CenterY", Ty::Integer, false),
            p("Radius", Ty::Integer, false),
            p("Color", Ty::Integer, false),
        ],
    );
    define_proc(
        checker,
        s::STD_GRAPH_APPLICATION_DRAW_TEXT,
        vec![
            p("App", types.application.clone(), false),
            p("X", Ty::Integer, false),
            p("Y", Ty::Integer, false),
            p("Text", Ty::String, false),
            p("Color", Ty::Integer, false),
        ],
    );
}
