//! `Std.Toml` semantic registration.
//!
//! **Documentation:** `docs/pascal/std/text/toml.md` (from the repository root).

use super::super::{define_func, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_std::std_symbols as s;
use std::sync::Arc;

/// Register the user-facing `Std.Toml` API.
///
/// **Documentation:** `docs/pascal/std/text/toml.md` (from the repository root).
pub(super) fn register_std_toml(checker: &mut Checker) {
    let toml_ref = Ty::Named(s::STD_TOML_VALUE.into());
    let variants = vec![
        EnumVariantTy {
            name: "String".into(),
            fields: vec![("Value".into(), Ty::String)],
            backing_value: None,
        },
        EnumVariantTy {
            name: "Integer".into(),
            fields: vec![("Value".into(), Ty::Integer)],
            backing_value: None,
        },
        EnumVariantTy {
            name: "Float".into(),
            fields: vec![("Value".into(), Ty::Real)],
            backing_value: None,
        },
        EnumVariantTy {
            name: "Boolean".into(),
            fields: vec![("Value".into(), Ty::Boolean)],
            backing_value: None,
        },
        EnumVariantTy {
            name: "Datetime".into(),
            fields: vec![("Value".into(), Ty::String)],
            backing_value: None,
        },
        EnumVariantTy {
            name: "Array".into(),
            fields: vec![("Items".into(), Ty::Array(Box::new(toml_ref.clone())))],
            backing_value: None,
        },
        EnumVariantTy {
            name: "Table".into(),
            fields: vec![(
                "Fields".into(),
                Ty::Dict(Box::new(Ty::String), Box::new(toml_ref)),
            )],
            backing_value: None,
        },
    ];
    let toml_ty = Ty::Enum(Arc::new(EnumTy {
        name: s::STD_TOML_VALUE.into(),
        variants: variants.clone(),
    }));

    checker.scopes.define(
        s::STD_TOML_VALUE,
        Symbol {
            ty: toml_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            task_bound: false,
        },
    );

    for variant in &variants {
        let qualified_name = format!("{}.{}", s::STD_TOML_VALUE, variant.name);
        checker.scopes.define(
            &qualified_name,
            Symbol {
                ty: toml_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumVariantConstructor,
                task_bound: false,
            },
        );
    }

    define_func(
        checker,
        s::STD_TOML_PARSE,
        vec![p("Text", Ty::String, false)],
        Ty::Result(Box::new(toml_ty.clone()), Box::new(Ty::String)),
    );
    define_func(
        checker,
        s::STD_TOML_STRINGIFY,
        vec![p("Value", toml_ty, false)],
        Ty::String,
    );
}
