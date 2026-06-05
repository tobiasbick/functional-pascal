//! `Std.Graph` semantic registration.
//!
//! **Documentation:** `docs/pascal/std/graph.md`, `docs/pascal/std/graph-app.md` (from the repository root).

mod application_api;
mod handlers;
mod host_api;
mod type_registration;

use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;
use fpas_std::{GRAPH_EVENT_KIND_VARIANTS, GRAPH_EXIT_REASON_VARIANTS};

pub(super) struct GraphTypes {
    application: Ty,
    size: Ty,
    event: Ty,
    application_handlers: Ty,
}

pub(super) struct GraphCallbackTypes {
    pub(super) on_key_pressed: Ty,
    pub(super) on_event: Ty,
    pub(super) on_resize: Ty,
    pub(super) on_paint: Ty,
    pub(super) on_close_requested: Ty,
    pub(super) on_idle: Ty,
    pub(super) on_exit: Ty,
}

/// Register the `Std.Graph` semantic surface.
pub(super) fn register_std_graph(checker: &mut Checker) {
    let application =
        type_registration::register_record_type(checker, s::STD_GRAPH_APPLICATION, Vec::new());
    let size = type_registration::register_record_type(
        checker,
        s::STD_GRAPH_SIZE,
        vec![
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );
    let key_event = lookup_required_type(
        checker,
        s::STD_CONSOLE_KEY_EVENT,
        "Std.Console.KeyEvent must be registered before Std.Graph (see loaded/mod.rs)",
    );
    let mouse_action = lookup_required_type(
        checker,
        s::STD_CONSOLE_MOUSE_ACTION,
        "Std.Console.MouseAction must be registered before Std.Graph (see loaded/mod.rs)",
    );
    let mouse_button = lookup_required_type(
        checker,
        s::STD_CONSOLE_MOUSE_BUTTON,
        "Std.Console.MouseButton must be registered before Std.Graph (see loaded/mod.rs)",
    );
    let event_kind = type_registration::register_enum_type(
        checker,
        s::STD_GRAPH_EVENT_KIND,
        GRAPH_EVENT_KIND_VARIANTS,
    );
    let event = type_registration::register_record_type(
        checker,
        s::STD_GRAPH_EVENT,
        vec![
            ("kind".into(), event_kind),
            ("size".into(), size.clone()),
            ("key".into(), key_event.clone()),
            ("mouse_action".into(), mouse_action),
            ("mouse_button".into(), mouse_button),
            ("mouse_x".into(), Ty::Integer),
            ("mouse_y".into(), Ty::Integer),
            ("wheel_x".into(), Ty::Integer),
            ("wheel_y".into(), Ty::Integer),
            ("shift".into(), Ty::Boolean),
            ("ctrl".into(), Ty::Boolean),
            ("alt".into(), Ty::Boolean),
            ("meta".into(), Ty::Boolean),
        ],
    );
    let exit_reason = type_registration::register_enum_type(
        checker,
        s::STD_GRAPH_EXIT_REASON,
        GRAPH_EXIT_REASON_VARIANTS,
    );

    let (application_handlers, callbacks) = handlers::register_application_handlers(
        checker,
        &application,
        &size,
        &event,
        &key_event,
        &exit_reason,
    );

    let types = GraphTypes {
        application,
        size,
        event,
        application_handlers,
    };

    application_api::register_application_api(checker, &types);
    host_api::register_host_api(checker, &types, &callbacks);
}

fn lookup_required_type(checker: &Checker, qualified_name: &str, message: &str) -> Ty {
    checker
        .scopes
        .lookup(qualified_name)
        .map(|symbol| symbol.ty.clone())
        .unwrap_or_else(|| unreachable!("{message}"))
}
