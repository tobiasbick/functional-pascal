use super::super::super::{define_func, define_proc, p};
use super::GraphTypes;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register the user-facing `Std.Graph.Application` calls.
///
/// **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).
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
    define_func(
        checker,
        s::STD_GRAPH_APPLICATION_SIZE,
        vec![p("App", types.application.clone(), false)],
        types.size.clone(),
    );
    define_func(
        checker,
        s::STD_GRAPH_APPLICATION_POLL_EVENT,
        vec![p("App", types.application.clone(), false)],
        Ty::Option(Box::new(types.event.clone())),
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
}
