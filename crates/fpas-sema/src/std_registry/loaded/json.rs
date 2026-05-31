//! `Std.Json` semantic registration.
//!
//! **Documentation:** `docs/pascal/std/json.md` (from the repository root).

use super::super::{define_func, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_std::std_symbols as s;

/// Register the user-facing `Std.Json` API.
///
/// **Documentation:** `docs/pascal/std/json.md` (from the repository root).
pub(super) fn register_std_json(checker: &mut Checker) {
    let json_ref = Ty::Named(s::STD_JSON_VALUE.into());
    let variants = vec![
        EnumVariantTy {
            name: "Null".into(),
            fields: vec![],
        },
        EnumVariantTy {
            name: "Bool".into(),
            fields: vec![("Value".into(), Ty::Boolean)],
        },
        EnumVariantTy {
            name: "Number".into(),
            fields: vec![("Value".into(), Ty::Real)],
        },
        EnumVariantTy {
            name: "String".into(),
            fields: vec![("Value".into(), Ty::String)],
        },
        EnumVariantTy {
            name: "Array".into(),
            fields: vec![("Items".into(), Ty::Array(Box::new(json_ref.clone())))],
        },
        EnumVariantTy {
            name: "Object".into(),
            fields: vec![(
                "Fields".into(),
                Ty::Dict(Box::new(Ty::String), Box::new(json_ref)),
            )],
        },
    ];
    let json_ty = Ty::Enum(EnumTy {
        name: s::STD_JSON_VALUE.into(),
        variants: variants.clone(),
    });

    checker.scopes.define(
        s::STD_JSON_VALUE,
        Symbol {
            ty: json_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );

    for variant in &variants {
        let qualified_name = format!("{}.{}", s::STD_JSON_VALUE, variant.name);
        let kind = if variant.fields.is_empty() {
            SymbolKind::EnumMember
        } else {
            SymbolKind::EnumVariantConstructor
        };
        checker.scopes.define(
            &qualified_name,
            Symbol {
                ty: json_ty.clone(),
                mutable: false,
                kind,
            },
        );
    }

    define_func(
        checker,
        s::STD_JSON_PARSE,
        vec![p("Text", Ty::String, false)],
        Ty::Result(Box::new(json_ty.clone()), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_JSON_STRINGIFY,
        vec![p("Value", json_ty, false)],
        Ty::String,
    );
}
