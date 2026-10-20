//! Native operational error declarations from the shared runtime catalog.

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_std::native_errors::{ErrorFieldType, NativeErrorFamily};
use std::sync::Arc;

pub(super) fn register(checker: &mut Checker, family: NativeErrorFamily) -> Ty {
    let host = if matches!(
        family,
        NativeErrorFamily::Fs | NativeErrorFamily::Proc | NativeErrorFamily::Crypto
    ) {
        Some(register(checker, NativeErrorFamily::Host))
    } else {
        None
    };
    let variants = family
        .variants()
        .iter()
        .map(|variant| EnumVariantTy {
            name: variant.name.into(),
            message: Some(variant.message.into()),
            backing_value: None,
            fields: variant
                .fields
                .iter()
                .map(|(name, ty)| {
                    (
                        name.to_string(),
                        match ty {
                            ErrorFieldType::String => Ty::String,
                            ErrorFieldType::Strings => Ty::Array(Box::new(Ty::String)),
                            ErrorFieldType::OptionalString => Ty::Option(Box::new(Ty::String)),
                            ErrorFieldType::OptionalInteger => Ty::Option(Box::new(Ty::Integer)),
                            ErrorFieldType::Integer => Ty::Integer,
                            ErrorFieldType::Host => host
                                .clone()
                                .unwrap_or(Ty::Named(NativeErrorFamily::Host.name().into())),
                        },
                    )
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let ty = Ty::Enum(Arc::new(EnumTy {
        is_error: true,
        name: family.name().into(),
        type_params: vec![],
        type_args: vec![],
        variants: variants.clone(),
    }));
    for (name, kind) in std::iter::once((family.name().to_string(), SymbolKind::Type)).chain(
        variants.iter().map(|variant| {
            (
                format!("{}.{}", family.name(), variant.name),
                SymbolKind::EnumVariantConstructor,
            )
        }),
    ) {
        checker.scopes.imports.define_metadata(
            &name,
            Symbol {
                constant: None,
                ty: ty.clone(),
                mutable: false,
                kind,
                task_bound: false,
            },
        );
    }
    ty
}
