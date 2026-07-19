use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_lexer::Span;
use fpas_parser::{EnumType, TypeDef};
use std::collections::HashSet;
use std::sync::Arc;

impl Checker {
    pub(super) fn check_enum_type_def(&mut self, td: &TypeDef, enum_ty: &EnumType) {
        if !self.scopes.define(
            &td.name,
            Symbol {
                ty: Ty::Named(td.name.clone()),
                mutable: false,
                kind: SymbolKind::Type,
                task_bound: false,
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

        let ty = Ty::Enum(Arc::new(EnumTy {
            name: td.name.clone(),
            variants: variants.clone(),
        }));

        for (member, variant) in enum_ty.members.iter().zip(variants.iter()) {
            let kind = if variant.fields.is_empty() {
                SymbolKind::EnumMember
            } else {
                SymbolKind::EnumVariantConstructor
            };
            let symbol = Symbol {
                ty: ty.clone(),
                mutable: false,
                kind,
                task_bound: false,
            };
            self.register_enum_variant_symbols(&td.name, &variant.name, symbol, member.span);
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing = Symbol {
                ty,
                mutable: false,
                kind: SymbolKind::Type,
                task_bound: false,
            };
        }
    }

    /// Register `Type.Variant` and, when unambiguous, a short `Variant` alias at program scope.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    fn register_enum_variant_symbols(
        &mut self,
        enum_name: &str,
        variant_name: &str,
        symbol: Symbol,
        span: Span,
    ) {
        let qualified = format!("{enum_name}.{variant_name}");
        if !self.scopes.define_in_root(&qualified, symbol.clone()) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate enum member `{qualified}`"),
                "Each enum member name must be unique in the program.",
                span,
            );
            return;
        }

        let short_key = canonical_symbol_name(variant_name);

        if let Some(candidates) = self.ambiguous_enum_variants.get(&short_key) {
            let mut updated = candidates.clone();
            if !updated
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&qualified))
            {
                updated.push(qualified.clone());
                self.ambiguous_enum_variants
                    .insert(short_key.clone(), updated);
            }
            return;
        }

        if let Some(existing_qualified) = self.enum_short_variant_keys.get(&short_key).cloned() {
            if existing_qualified.eq_ignore_ascii_case(&qualified) {
                return;
            }

            self.scopes.remove_from_root(variant_name);
            self.enum_short_variant_keys.remove(&short_key);
            self.ambiguous_enum_variants
                .insert(short_key, vec![existing_qualified, qualified.clone()]);
            return;
        }

        if self.scopes.lookup(variant_name).is_some() {
            return;
        }

        if self.scopes.define_in_root(variant_name, symbol) {
            self.enum_short_variant_keys.insert(short_key, qualified);
        }
    }
}
