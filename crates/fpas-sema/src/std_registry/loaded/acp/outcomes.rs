//! SDK initialization outcomes are detached values, separate from wire payloads.

use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::{EnumTy, EnumVariantTy, Ty},
};
use std::sync::Arc;

pub(super) fn register(checker: &mut Checker) {
    let response = |checker: &Checker, module: &str| {
        checker
            .scopes
            .imports
            .lookup_metadata(&format!("{module}.InitializeResponse"))
            .expect("registered initialization payload")
            .ty
            .clone()
    };
    let selected = enumeration(
        checker,
        "SelectedInitializeResponse",
        vec![("V1", response(checker, "Std.Acp.Types"))],
    );
    let mismatch = super::super::type_registration::register_record_type(
        checker,
        "Std.Acp.V2.InitializationVersionMismatch",
        vec![
            ("Requested".into(), Ty::Integer),
            ("Selected".into(), Ty::Integer),
            ("Response".into(), selected),
        ],
    );
    enumeration(
        checker,
        "InitializeOutcome",
        vec![
            ("Initialized", response(checker, "Std.Acp.V2")),
            ("LowerVersion", mismatch),
        ],
    );
}

fn enumeration(checker: &mut Checker, name: &str, variants: Vec<(&str, Ty)>) -> Ty {
    let name = format!("Std.Acp.V2.{name}");
    let variants = variants
        .into_iter()
        .map(|(name, ty)| EnumVariantTy {
            name: name.into(),
            fields: vec![("Payload".into(), ty)],
            message: None,
            backing_value: None,
        })
        .collect::<Vec<_>>();
    let ty = Ty::Enum(Arc::new(EnumTy {
        name: name.clone(),
        is_error: false,
        type_params: vec![],
        type_args: vec![],
        variants: variants.clone(),
    }));
    for (name, kind) in
        std::iter::once((name.clone(), SymbolKind::Type)).chain(variants.iter().map(|variant| {
            (
                format!("{name}.{}", variant.name),
                SymbolKind::EnumVariantConstructor,
            )
        }))
    {
        checker.scopes.imports.define_metadata(
            &name,
            Symbol {
                ty: ty.clone(),
                constant: None,
                kind,
                mutable: false,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }
    ty
}
