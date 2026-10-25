//! Complete, detached ACP protocol types and value codecs.

mod codecs;
mod declarations;
mod outcomes;

use std::sync::Arc;

use super::super::{define_func, p};
use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::{EnumTy, EnumVariantTy, GenericParamDef, Ty},
};
use fpas_std::{native_errors::NativeErrorFamily, std_symbols as s};

pub(super) fn register(checker: &mut Checker) {
    let raw = super::native_records::register(checker, "Std.Acp.Types.RawJson");
    let unsigned = super::native_records::register(checker, "Std.Acp.Types.UInt64");
    let error = super::errors::register(checker, NativeErrorFamily::Acp);
    field(checker);
    declarations::register(checker);
    outcomes::register(checker);
    codecs::register(checker);
    define_func(
        checker,
        s::STD_ACP_RAW_JSON_MEMBER,
        vec![p("Value", raw.clone()), p("Name", Ty::String)],
        Ty::Result(
            Box::new(Ty::Option(Box::new(raw.clone()))),
            Box::new(error.clone()),
        ),
    );
    for (name, parameter, input, output) in [
        (s::STD_ACP_PARSE_RAW_JSON, "Text", Ty::String, raw.clone()),
        (s::STD_ACP_RAW_JSON_TEXT, "Value", raw, Ty::String),
        (
            s::STD_ACP_PARSE_UINT64,
            "Text",
            Ty::String,
            unsigned.clone(),
        ),
        (
            s::STD_ACP_UINT64_TEXT,
            "Value",
            unsigned.clone(),
            Ty::String,
        ),
        (
            s::STD_ACP_UINT64_FROM_INTEGER,
            "Value",
            Ty::Integer,
            unsigned.clone(),
        ),
        (s::STD_ACP_UINT64_TO_INTEGER, "Value", unsigned, Ty::Integer),
    ] {
        define_func(
            checker,
            name,
            vec![p(parameter, input)],
            Ty::Result(Box::new(output), Box::new(error.clone())),
        );
    }
}

fn field(checker: &mut Checker) {
    const NAME: &str = "Std.Acp.Types.Field";
    let variants = vec![
        EnumVariantTy {
            name: "Missing".into(),
            message: None,
            backing_value: None,
            fields: vec![],
        },
        EnumVariantTy {
            name: "NullValue".into(),
            message: None,
            backing_value: None,
            fields: vec![],
        },
        EnumVariantTy {
            name: "Value".into(),
            message: None,
            backing_value: None,
            fields: vec![("Value".into(), Ty::GenericParam("T".into(), None))],
        },
    ];
    let ty = Ty::Enum(Arc::new(EnumTy {
        name: NAME.into(),
        is_error: false,
        type_params: vec![GenericParamDef {
            name: "T".into(),
            constraint: None,
        }],
        type_args: vec![],
        variants: variants.clone(),
    }));
    for (name, kind) in std::iter::once((NAME.to_string(), SymbolKind::Type)).chain(
        variants.iter().map(|variant| {
            (
                format!("{NAME}.{}", variant.name),
                if variant.fields.is_empty() {
                    SymbolKind::EnumMember
                } else {
                    SymbolKind::EnumVariantConstructor
                },
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
                capability: crate::TransferCapability::Transferable,
            },
        );
    }
}
