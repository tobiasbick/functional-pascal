//! `Std.Encoding.Toml` semantic registration.
//!
//! **Documentation:** `docs/pascal/std/text/toml.md` (from the repository root).

use super::super::{define_func, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_builtins::std_symbols as s;
use std::sync::Arc;

/// Register the user-facing `Std.Encoding.Toml` API.
///
/// **Documentation:** `docs/pascal/std/text/toml.md` (from the repository root).
pub(super) fn register_std_toml(checker: &mut Checker) {
    let error_ty = super::errors::register(
        checker,
        fpas_builtins::native_errors::NativeErrorFamily::Toml,
    );
    let toml_ref = Ty::Named(s::STD_TOML_VALUE.into());
    let variants = vec![
        EnumVariantTy {
            message: None,
            name: "String".into(),
            fields: vec![("Value".into(), Ty::String)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Integer".into(),
            fields: vec![("Value".into(), Ty::Integer)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Float".into(),
            fields: vec![("Value".into(), Ty::Real)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Boolean".into(),
            fields: vec![("Value".into(), Ty::Boolean)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Datetime".into(),
            fields: vec![("Value".into(), Ty::String)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "ArrayValue".into(),
            fields: vec![("Items".into(), Ty::Array(Box::new(toml_ref.clone())))],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Table".into(),
            fields: vec![(
                "Fields".into(),
                Ty::Dict(Box::new(Ty::String), Box::new(toml_ref)),
            )],
            backing_value: None,
        },
    ];
    let toml_ty = Ty::Enum(Arc::new(EnumTy {
        is_error: false,
        name: s::STD_TOML_VALUE.into(),
        type_params: Vec::new(),
        type_args: Vec::new(),
        variants: variants.clone(),
    }));

    checker.scopes.imports.define_metadata(
        s::STD_TOML_VALUE,
        Symbol {
            constant: None,
            ty: toml_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            capability: crate::TransferCapability::Transferable,
        },
    );

    for variant in &variants {
        let qualified_name = format!("{}.{}", s::STD_TOML_VALUE, variant.name);
        checker.scopes.imports.define_metadata(
            &qualified_name,
            Symbol {
                constant: None,
                ty: toml_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumVariantConstructor,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }

    define_func(
        checker,
        s::STD_TOML_PARSE,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(toml_ty.clone()), Box::new(error_ty)),
    );
    define_func(
        checker,
        s::STD_TOML_STRINGIFY,
        vec![p("Value", toml_ty)],
        Ty::String,
    );
}
