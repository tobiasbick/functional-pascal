//! `Std.Json` semantic registration.
//!
//! **Documentation:** `docs/pascal/std/text/json.md` (from the repository root).

use super::super::{define_func, p};
use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_std::std_symbols as s;
use std::sync::Arc;

/// Register the user-facing `Std.Json` API.
///
/// **Documentation:** `docs/pascal/std/text/json.md` (from the repository root).
pub(super) fn register_std_json(checker: &mut Checker) {
    let error_ty =
        super::errors::register(checker, fpas_std::native_errors::NativeErrorFamily::Json);
    let json_ref = Ty::Named(s::STD_JSON_VALUE.into());
    let variants = vec![
        EnumVariantTy {
            message: None,
            name: "NullValue".into(),
            fields: vec![],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Bool".into(),
            fields: vec![("Value".into(), Ty::Boolean)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Number".into(),
            fields: vec![("Value".into(), Ty::Real)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "String".into(),
            fields: vec![("Value".into(), Ty::String)],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "ArrayValue".into(),
            fields: vec![("Items".into(), Ty::Array(Box::new(json_ref.clone())))],
            backing_value: None,
        },
        EnumVariantTy {
            message: None,
            name: "Object".into(),
            fields: vec![(
                "Fields".into(),
                Ty::Dict(Box::new(Ty::String), Box::new(json_ref)),
            )],
            backing_value: None,
        },
    ];
    let json_ty = Ty::Enum(Arc::new(EnumTy {
        is_error: false,
        name: s::STD_JSON_VALUE.into(),
        type_params: Vec::new(),
        type_args: Vec::new(),
        variants: variants.clone(),
    }));

    checker.scopes.imports.define_metadata(
        s::STD_JSON_VALUE,
        Symbol {
            constant: None,
            ty: json_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            capability: crate::TransferCapability::Transferable,
        },
    );

    for variant in &variants {
        let qualified_name = format!("{}.{}", s::STD_JSON_VALUE, variant.name);
        let kind = if variant.fields.is_empty() {
            SymbolKind::EnumMember
        } else {
            SymbolKind::EnumVariantConstructor
        };
        checker.scopes.imports.define_metadata(
            &qualified_name,
            Symbol {
                constant: None,
                ty: json_ty.clone(),
                mutable: false,
                kind,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }

    define_func(
        checker,
        s::STD_JSON_PARSE,
        vec![p("Text", Ty::String)],
        Ty::Result(Box::new(json_ty.clone()), Box::new(error_ty.clone())),
    );
    define_func(
        checker,
        s::STD_JSON_STRINGIFY,
        vec![p("Value", json_ty)],
        Ty::String,
    );
}
