//! `Std.Tui` scene-graph introspection type registration.

use crate::check::Checker;
use crate::std_registry::loaded::type_registration;
use crate::types::Ty;
use fpas_std::TUI_VIEW_KIND_VARIANTS;
use fpas_std::std_symbols as s;

/// Registered types returned by scene-graph query functions.
pub(super) struct TuiIntrospectionTypes {
    pub(super) view_state: Ty,
    pub(super) view_options: Ty,
    pub(super) view_layout: Ty,
    pub(super) resolved_view: Ty,
    pub(super) view_kind: Ty,
    pub(super) view_snapshot: Ty,
}

/// Register the read-only scene-graph records and view-kind enum.
pub(super) fn register(checker: &mut Checker, view_id: &Ty, rect: &Ty) -> TuiIntrospectionTypes {
    let view_state = type_registration::register_record_type(
        checker,
        s::STD_TUI_VIEW_STATE,
        vec![
            ("visible".into(), Ty::Boolean),
            ("enabled".into(), Ty::Boolean),
            ("focused".into(), Ty::Boolean),
            ("active".into(), Ty::Boolean),
            ("exposed".into(), Ty::Boolean),
        ],
    );
    let view_options = type_registration::register_record_type(
        checker,
        s::STD_TUI_VIEW_OPTIONS,
        vec![
            ("selectable".into(), Ty::Boolean),
            ("tabStop".into(), Ty::Boolean),
            ("preProcess".into(), Ty::Boolean),
            ("postProcess".into(), Ty::Boolean),
            ("clipChildren".into(), Ty::Boolean),
        ],
    );
    let view_layout = type_registration::register_record_type(
        checker,
        s::STD_TUI_VIEW_LAYOUT,
        vec![
            ("anchorLeft".into(), Ty::Boolean),
            ("anchorTop".into(), Ty::Boolean),
            ("anchorRight".into(), Ty::Boolean),
            ("anchorBottom".into(), Ty::Boolean),
            ("marginLeft".into(), Ty::Integer),
            ("marginTop".into(), Ty::Integer),
            ("marginRight".into(), Ty::Integer),
            ("marginBottom".into(), Ty::Integer),
        ],
    );
    let resolved_view = type_registration::register_record_type(
        checker,
        s::STD_TUI_RESOLVED_VIEW,
        vec![
            ("rect".into(), rect.clone()),
            ("clip".into(), Ty::Option(Box::new(rect.clone()))),
            ("state".into(), view_state.clone()),
            ("options".into(), view_options.clone()),
        ],
    );
    let view_kind = type_registration::register_enum_type(
        checker,
        s::STD_TUI_VIEW_KIND,
        TUI_VIEW_KIND_VARIANTS,
    );
    let view_snapshot = type_registration::register_record_type(
        checker,
        s::STD_TUI_VIEW_SNAPSHOT,
        vec![
            ("id".into(), view_id.clone()),
            ("parent".into(), Ty::Option(Box::new(view_id.clone()))),
            ("children".into(), Ty::Array(Box::new(view_id.clone()))),
            ("resolved".into(), resolved_view.clone()),
            ("kind".into(), view_kind.clone()),
        ],
    );

    TuiIntrospectionTypes {
        view_state,
        view_options,
        view_layout,
        resolved_view,
        view_kind,
        view_snapshot,
    }
}
