use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{EnumType, TypeDef};
use std::collections::HashSet;

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

        let mut seen_variants = HashSet::new();
        let mut variants = Vec::new();
        for member in &enum_ty.members {
            if !seen_variants.insert(canonical_symbol_name(&member.name)) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate enum member `{}`", member.name),
                    "Each enum member name must be unique within the enum.",
                    member.span,
                );
                continue;
            }

            let mut seen_fields = HashSet::new();
            let mut fields = Vec::new();
            for field in &member.fields {
                if !seen_fields.insert(canonical_symbol_name(&field.name)) {
                    self.error_with_code(
                        SEMA_DUPLICATE_DECLARATION,
                        format!("Duplicate enum field `{}`", field.name),
                        "Each associated-data field name must be unique within the enum member.",
                        field.span,
                    );
                    continue;
                }
                fields.push((field.name.clone(), self.resolve_type_expr(&field.type_expr)));
            }

            variants.push(EnumVariantTy {
                name: member.name.clone(),
                fields,
            });
        }

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
