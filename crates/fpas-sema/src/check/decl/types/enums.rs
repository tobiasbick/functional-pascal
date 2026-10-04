use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_ENUM_BACKING_VALUE_EXHAUSTED};
use fpas_lexer::Span;
use fpas_parser::{EnumType, TypeDef};
use std::collections::HashSet;
use std::sync::Arc;

impl Checker {
    pub(super) fn check_enum_type_def(&mut self, td: &TypeDef, enum_ty: &EnumType) {
        if !self.has_collected_type(td)
            && !self.scopes.define(
                &td.name,
                Symbol {
                    ty: Ty::Named(td.name.clone()),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            )
        {
            self.define_type_symbol(td, Ty::Error);
            return;
        }

        let backing_values = self.enum_backing_values(&td.name, enum_ty);

        let mut seen_variants = HashSet::new();
        let mut variants = Vec::new();
        let mut variant_spans = Vec::new();
        for (member, backing_value) in enum_ty.members.iter().zip(backing_values) {
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
                backing_value,
            });
            variant_spans.push(member.span);
        }

        let ty = Ty::Enum(Arc::new(EnumTy {
            name: td.name.clone(),
            type_params: Self::resolve_type_params(&td.type_params),
            type_args: Vec::new(),
            variants: variants.clone(),
        }));

        for (variant, span) in variants.iter().zip(variant_spans) {
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
            self.register_enum_variant_symbols(&td.name, &variant.name, symbol, span);
        }

        if let Some(existing) = self.scopes.lookup_root_mut(&td.name) {
            *existing = Symbol {
                ty,
                mutable: false,
                kind: SymbolKind::Type,
                task_bound: false,
            };
        }
    }

    fn enum_backing_values(&mut self, enum_name: &str, enum_ty: &EnumType) -> Vec<Option<i64>> {
        if enum_ty
            .members
            .iter()
            .any(|member| !member.fields.is_empty())
        {
            return vec![None; enum_ty.members.len()];
        }

        let mut next_value = Some(0_i64);
        let mut values = Vec::with_capacity(enum_ty.members.len());
        for member in &enum_ty.members {
            let backing = match member.value {
                Some(value) => value,
                None => {
                    let Some(value) = next_value else {
                        self.error_with_code(
                            SEMA_ENUM_BACKING_VALUE_EXHAUSTED,
                            format!(
                                "Implicit backing value for enum member `{enum_name}.{}` exceeds the integer range",
                                member.name
                            ),
                            "Assign an explicit integer backing value to this member, or choose a preceding value below 9223372036854775807.",
                            member.span,
                        );
                        values.push(None);
                        continue;
                    };
                    value
                }
            };
            values.push(Some(backing));
            next_value = backing.checked_add(1);
        }
        values
    }

    /// Register only the qualified `Type.Variant` symbol.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`.
    fn register_enum_variant_symbols(
        &mut self,
        enum_name: &str,
        variant_name: &str,
        symbol: Symbol,
        span: Span,
    ) {
        let qualified = format!("{enum_name}.{variant_name}");
        if !self.scopes.define_in_root(&qualified, symbol) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate enum member `{qualified}`"),
                "Each enum member name must be unique in its declaring type.",
                span,
            );
        }
    }
}
