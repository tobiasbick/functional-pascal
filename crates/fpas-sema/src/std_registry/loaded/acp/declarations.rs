//! Native declarations use the same SDK-derived shapes as the runtime converter.

use crate::{
    check::Checker,
    scope::{Symbol, SymbolKind},
    types::{DistinctTy, EnumTy, EnumVariantTy, RecordTy, Ty},
};
use fpas_acp_model::{
    binding::{self, Catalog, Shape, Type, Unknown},
    schema::Version,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[expect(
    clippy::expect_used,
    reason = "Both pinned SDK declaration graphs are validated by the ACP schema tests"
)]
pub(super) fn register(checker: &mut Checker) {
    for version in [Version::V1, Version::V2] {
        let catalog = binding::catalog(version).expect("pinned ACP declaration graph");
        let mut registry = Registry {
            checker,
            catalog,
            types: HashMap::new(),
            active: HashSet::new(),
        };
        for key in catalog.declarations.keys() {
            registry.named(key);
        }
    }
    for name in [
        "Connection",
        "RequestToken",
        "Responder",
        "MessageEndpoint",
        "McpEndpoint",
    ] {
        super::super::type_registration::register_opaque_type(
            checker,
            &format!("Std.Ai.Acp.{name}"),
        );
    }
    for name in [
        "ReadTextFileRequest",
        "ReadTextFileResponse",
        "WriteTextFileRequest",
        "WriteTextFileResponse",
        "CreateTerminalRequest",
        "CreateTerminalResponse",
        "TerminalOutputRequest",
        "TerminalOutputResponse",
        "WaitForTerminalExitRequest",
        "WaitForTerminalExitResponse",
        "KillTerminalRequest",
        "KillTerminalResponse",
        "ReleaseTerminalRequest",
        "ReleaseTerminalResponse",
    ] {
        if let Some(symbol) = checker
            .scopes
            .imports
            .lookup_metadata(&format!("Std.Ai.Acp.Types.{name}"))
        {
            let symbol = symbol.clone();
            checker
                .scopes
                .imports
                .define_metadata(&format!("Std.Ai.Acp.Host.{name}"), symbol);
        }
    }
}

struct Registry<'a> {
    checker: &'a mut Checker,
    catalog: &'static Catalog,
    types: HashMap<String, Ty>,
    active: HashSet<String>,
}

impl Registry<'_> {
    fn named(&mut self, key: &str) -> Ty {
        let declaration = &self.catalog.declarations[key];
        if let Some(ty) = self.types.get(key) {
            return ty.clone();
        }
        if let Some(symbol) = self
            .checker
            .scopes
            .imports
            .lookup_metadata(&declaration.name)
            && declaration.name == "Std.Ai.Acp.Types.ProtocolError"
        {
            return symbol.ty.clone();
        }
        if !self.active.insert(key.into()) {
            return Ty::Named(declaration.name.clone());
        }
        let ty = match &declaration.shape {
            Shape::Scalar(Type::String) => Ty::Distinct(Arc::new(DistinctTy {
                name: declaration.name.clone(),
                owner_unit: Some(declaration.module.into()),
                underlying: Ty::String,
            })),
            Shape::Scalar(ty) => self.ty(ty),
            Shape::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|field| (field.name.clone(), self.ty(&field.ty)))
                    .collect();
                self.record(&declaration.name, fields)
            }
            Shape::Enum { variants, unknown } => {
                let mut variants = variants
                    .iter()
                    .map(|variant| EnumVariantTy {
                        name: variant.name.clone(),
                        message: None,
                        backing_value: None,
                        fields: variant
                            .payload
                            .as_ref()
                            .map(|ty| ("Payload".into(), self.ty(ty)))
                            .into_iter()
                            .collect(),
                    })
                    .collect::<Vec<_>>();
                if let Some(unknown) = unknown {
                    let fields = match unknown {
                        Unknown::String => vec![("Value".into(), Ty::String)],
                        Unknown::Tagged(_) => vec![
                            ("Tag".into(), Ty::String),
                            ("Payload".into(), self.shared("RawJson")),
                        ],
                    };
                    variants.push(EnumVariantTy {
                        name: "Unknown".into(),
                        message: None,
                        backing_value: None,
                        fields,
                    });
                }
                Ty::Enum(Arc::new(EnumTy {
                    name: declaration.name.clone(),
                    is_error: false,
                    type_params: vec![],
                    type_args: vec![],
                    variants,
                }))
            }
            Shape::Extension { response } => {
                let mut fields = Vec::new();
                if !response {
                    fields.push(("Method".into(), Ty::String));
                }
                fields.push((
                    if *response { "ResultValue" } else { "Params" }.into(),
                    self.shared("RawJson"),
                ));
                self.record(&declaration.name, fields)
            }
        };
        self.define(&declaration.name, ty.clone(), SymbolKind::Type);
        if let Ty::Enum(enumeration) = &ty {
            for variant in &enumeration.variants {
                self.define(
                    &format!("{}.{}", declaration.name, variant.name),
                    ty.clone(),
                    if variant.fields.is_empty() {
                        SymbolKind::EnumMember
                    } else {
                        SymbolKind::EnumVariantConstructor
                    },
                );
            }
        }
        self.active.remove(key);
        self.types.insert(key.into(), ty.clone());
        ty
    }

    #[expect(
        clippy::expect_used,
        reason = "ACP registration installs shared foundation types before payload declarations"
    )]
    fn shared(&self, name: &str) -> Ty {
        self.checker
            .scopes
            .imports
            .lookup_metadata(&format!("Std.Ai.Acp.Types.{name}"))
            .expect("registered ACP foundation")
            .ty
            .clone()
    }

    fn ty(&mut self, ty: &Type) -> Ty {
        match ty {
            Type::Boolean => Ty::Boolean,
            Type::Integer { .. } => Ty::Integer,
            Type::Real => Ty::Real,
            Type::String => Ty::String,
            Type::UInt64 => self.shared("UInt64"),
            Type::Raw { .. } => self.shared("RawJson"),
            Type::Array(inner) => Ty::Array(Box::new(self.ty(inner))),
            Type::Dictionary(inner) => Ty::Dict(Box::new(Ty::String), Box::new(self.ty(inner))),
            Type::Option(inner) => Ty::Option(Box::new(self.ty(inner))),
            Type::Field(inner) => {
                let inner = self.ty(inner);
                let Ty::Enum(field) = self.shared("Field") else {
                    unreachable!("native Field declaration");
                };
                Ty::Enum(Arc::new(field.instantiate(vec![inner])))
            }
            Type::Named(key) => self.named(key),
        }
    }

    fn record(&self, name: &str, fields: Vec<(String, Ty)>) -> Ty {
        Ty::Record(Arc::new(RecordTy {
            name: name.into(),
            owner_unit: None,
            type_params: vec![],
            type_args: vec![],
            private_members: vec![],
            fields,
            methods: vec![],
            release_method: None,
            static_functions: vec![],
            static_procedures: vec![],
        }))
    }

    fn define(&mut self, name: &str, ty: Ty, kind: SymbolKind) {
        self.checker.scopes.imports.define_metadata(
            name,
            Symbol {
                constant: None,
                ty,
                kind,
                mutable: false,
                capability: crate::TransferCapability::Transferable,
            },
        );
    }
}
