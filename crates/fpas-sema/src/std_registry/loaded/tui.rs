//! `Std.Tui` semantic registration.
//!
//! `Std.Tui.TuiEvent.key` uses `Std.Console.KeyEvent` (registered by [`super::console::register_std_console_key_api`] when needed).
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

use super::super::{define_func, define_proc, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy, Ty};
use fpas_std::TUI_EVENT_KIND_VARIANTS;
use fpas_std::std_symbols as s;

fn register_enum_type(checker: &mut Checker, qualified_name: &str, variants: &[&str]) -> Ty {
    let variants: Vec<EnumVariantTy> = variants
        .iter()
        .map(|variant| EnumVariantTy {
            name: (*variant).to_string(),
            fields: vec![],
        })
        .collect();
    let member_names: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
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

    for member in &member_names {
        let qualified = format!("{qualified_name}.{member}");
        checker.scopes.define(
            &qualified,
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

pub(super) fn register_std_tui(checker: &mut Checker) {
    let application_ty = register_record_type(checker, s::STD_TUI_APPLICATION, Vec::new());
    let size_ty = register_record_type(
        checker,
        s::STD_TUI_SIZE,
        vec![
            ("width".into(), Ty::Integer),
            ("height".into(), Ty::Integer),
        ],
    );

    let key_event_ty = match checker.scopes.lookup(s::STD_CONSOLE_KEY_EVENT) {
        Some(sym) => sym.ty.clone(),
        None => unreachable!(
            "Std.Console.KeyEvent must be registered before Std.Tui (see loaded/mod.rs)"
        ),
    };

    let event_kind_ty = register_enum_type(checker, s::STD_TUI_EVENT_KIND, TUI_EVENT_KIND_VARIANTS);
    let event_ty = register_record_type(
        checker,
        s::STD_TUI_EVENT,
        vec![
            ("kind".into(), event_kind_ty),
            ("key".into(), key_event_ty),
            ("size".into(), size_ty.clone()),
        ],
    );

    define_func(
        checker,
        s::STD_TUI_APPLICATION_OPEN,
        vec![],
        application_ty.clone(),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_CLOSE,
        vec![p("App", application_ty.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_SIZE,
        vec![p("App", application_ty.clone(), false)],
        size_ty,
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_READ_EVENT,
        vec![p("App", application_ty.clone(), false)],
        event_ty.clone(),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_READ_EVENT_TIMEOUT,
        vec![
            p("App", application_ty.clone(), false),
            p("Milliseconds", Ty::Integer, false),
        ],
        Ty::Option(Box::new(event_ty.clone())),
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_POLL_EVENT,
        vec![p("App", application_ty.clone(), false)],
        Ty::Option(Box::new(event_ty)),
    );
    define_proc(
        checker,
        s::STD_TUI_APPLICATION_REQUEST_REDRAW,
        vec![p("App", application_ty.clone(), false)],
    );
    define_func(
        checker,
        s::STD_TUI_APPLICATION_REDRAW_PENDING,
        vec![p("App", application_ty, false)],
        Ty::Boolean,
    );
}
