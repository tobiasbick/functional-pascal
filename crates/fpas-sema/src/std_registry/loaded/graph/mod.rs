//! `Std.Graph` semantic registration.
//!
//! `Std.Graph.Event` reuses `Std.Console.KeyEvent`, `Std.Console.MouseAction`, and
//! `Std.Console.MouseButton` (registered by [`super::console::register_std_console_key_api`]
//! when needed).
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

mod application_api;

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy, Ty};
use fpas_std::GRAPH_EVENT_KIND_VARIANTS;
use fpas_std::std_symbols as s;

struct GraphTypes {
    application: Ty,
    size: Ty,
    event: Ty,
}

/// Register the `Std.Graph` semantic surface for the Phase 1 MVP.
///
/// **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).
pub(super) fn register_std_graph(checker: &mut Checker) {
    let application = register_record_type(checker, s::STD_GRAPH_APPLICATION, Vec::new());
    let size = register_record_type(
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
    let event_kind =
        register_enum_type(checker, s::STD_GRAPH_EVENT_KIND, GRAPH_EVENT_KIND_VARIANTS);
    let event = register_record_type(
        checker,
        s::STD_GRAPH_EVENT,
        vec![
            ("kind".into(), event_kind),
            ("size".into(), size.clone()),
            ("key".into(), key_event),
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

    let types = GraphTypes {
        application,
        size,
        event,
    };
    application_api::register_application_api(checker, &types);
}

fn register_enum_type(checker: &mut Checker, qualified_name: &str, variants: &[&str]) -> Ty {
    let variants: Vec<EnumVariantTy> = variants
        .iter()
        .map(|variant| EnumVariantTy {
            name: (*variant).to_string(),
            fields: vec![],
        })
        .collect();
    let member_names: Vec<String> = variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect();
    let enum_ty = Ty::Enum(EnumTy {
        name: qualified_name.into(),
        variants,
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: enum_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );

    for member_name in &member_names {
        let qualified_member = format!("{qualified_name}.{member_name}");
        checker.scopes.define(
            &qualified_member,
            Symbol {
                ty: enum_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumMember,
            },
        );
    }

    enum_ty
}

fn register_record_type(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
) -> Ty {
    let record_ty = Ty::Record(RecordTy {
        name: qualified_name.into(),
        fields,
        methods: Vec::new(),
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: record_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );
    record_ty
}

fn lookup_required_type(checker: &Checker, qualified_name: &str, message: &str) -> Ty {
    checker
        .scopes
        .lookup(qualified_name)
        .map(|symbol| symbol.ty.clone())
        .unwrap_or_else(|| unreachable!("{message}"))
}
