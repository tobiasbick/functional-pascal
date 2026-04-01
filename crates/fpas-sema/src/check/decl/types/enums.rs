use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_parser::{EnumType, TypeDef};

impl Checker {
    pub(super) fn check_enum_type_def(&mut self, td: &TypeDef, enum_ty: &EnumType) {
        if !self.scopes.define(
            &td.name,
            Symbol {
                ty: Ty::Named(td.name.clone()),
                mutable: false,
                kind: SymbolKind::Type,
            },
        ) {
            self.define_type_symbol(td, Ty::Error);
            return;
        }

        let variants: Vec<EnumVariantTy> = enum_ty
            .members
            .iter()
            .map(|member| {
                let fields = member
                    .fields
                    .iter()
                    .map(|field| {
                        (
                            field.name.clone(),
                            self.resolve_type_expr(&field.type_expr),
                        )
                    })
                    .collect();
                EnumVariantTy {
                    name: member.name.clone(),
                    fields,
                }
            })
            .collect();

        let ty = Ty::Enum(EnumTy {
            name: td.name.clone(),
            variants: variants.clone(),
        });

        for variant in &variants {
            let kind = if variant.fields.is_empty() {
                SymbolKind::EnumMember
            } else {
                SymbolKind::EnumVariantConstructor
            };
            let symbol = Symbol {
                ty: ty.clone(),
                mutable: false,
                kind,
            };
            self.scopes.define(&variant.name, symbol.clone());
            let qualified = format!("{}.{}", td.name, variant.name);
            self.scopes.define(&qualified, symbol);
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing = Symbol {
                ty,
                mutable: false,
                kind: SymbolKind::Type,
            };
        }
    }
}
