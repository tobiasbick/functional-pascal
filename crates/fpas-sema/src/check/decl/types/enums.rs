use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use fpas_diagnostics::codes::{
    SEMA_BACKING_VALUE_IN_DATA_ENUM, SEMA_DUPLICATE_DECLARATION, SEMA_DUPLICATE_ENUM_BACKING_VALUE,
    SEMA_ENUM_BACKING_VALUE_EXHAUSTED,
};
use fpas_lexer::Span;
use fpas_parser::{EnumType, TypeDef};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

impl Checker {
    /// Collect enum payloads and register qualified variants before ordered bodies.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    pub(super) fn check_enum_type_def(&mut self, td: &TypeDef, enum_ty: &EnumType) {
        if !self.has_collected_type(td)
            && !self.scopes.define(
                &td.name,
                Symbol {
                    constant: None,
                    ty: Ty::Named(td.name.clone()),
                    mutable: false,
                    kind: SymbolKind::Type,
                    capability: crate::TransferCapability::Transferable,
                },
            )
        {
            self.define_type_symbol(td, Ty::Error);
            return;
        }

        let backing_values = self.enum_backing_values(&td.name, enum_ty);

        let (variants, variant_spans) =
            self.with_type_params(&td.type_params, td.span, |checker| {
                let mut seen_variants = HashSet::new();
                let mut variants = Vec::new();
                let mut variant_spans = Vec::new();
                for (member, backing_value) in enum_ty.members.iter().zip(backing_values) {
                    if !seen_variants.insert(canonical_symbol_name(&member.name)) {
                        checker.error_with_code(
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
                            checker.error_with_code(
                        SEMA_DUPLICATE_DECLARATION,
                        format!("Duplicate enum field `{}`", field.name),
                        "Each associated-data field name must be unique within the enum member.",
                        field.span,
                    );
                            continue;
                        }
                        fields.push((
                            field.name.clone(),
                            checker.resolve_type_expr(&field.type_expr),
                        ));
                    }

                    variants.push(EnumVariantTy {
                        message: member.message.clone(),
                        name: member.name.clone(),
                        fields,
                        backing_value,
                    });
                    variant_spans.push(member.span);
                }
                (variants, variant_spans)
            });

        let ty = Ty::Enum(Arc::new(EnumTy {
            is_error: enum_ty.is_error,
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
                constant: None,
                ty: ty.clone(),
                mutable: false,
                kind,
                capability: crate::TransferCapability::Transferable,
            };
            self.register_enum_variant_symbols(&td.name, &variant.name, symbol, span);
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing = Symbol {
                constant: None,
                ty,
                mutable: false,
                kind: SymbolKind::Type,
                capability: crate::TransferCapability::Transferable,
            };
        }
    }

    // One integer identifies each simple variant; data variants use their tags instead.
    fn enum_backing_values(&mut self, enum_name: &str, enum_ty: &EnumType) -> Vec<Option<i64>> {
        if enum_ty.is_error {
            return vec![None; enum_ty.members.len()];
        }
        if enum_ty
            .members
            .iter()
            .any(|member| !member.fields.is_empty())
        {
            for member in &enum_ty.members {
                if let Some(value) = member.value {
                    self.error_with_code(
                        SEMA_BACKING_VALUE_IN_DATA_ENUM,
                        format!(
                            "Enum member `{enum_name}.{}` declares backing value {value} in an enum with associated data",
                            member.name
                        ),
                        format!(
                            "Remove the backing value: write `{};`. Only enums without associated-data fields may declare integer backing values.",
                            member.name
                        ),
                        member.span,
                    );
                }
            }
            return vec![None; enum_ty.members.len()];
        }

        let mut next_value = Some(0_i64);
        let mut values = Vec::with_capacity(enum_ty.members.len());
        let mut seen_values = HashMap::new();
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
            if let Some(previous) = seen_values.get(&backing) {
                self.error_with_code(
                    SEMA_DUPLICATE_ENUM_BACKING_VALUE,
                    format!(
                        "Enum members `{enum_name}.{previous}` and `{enum_name}.{}` have the same backing value {backing}",
                        member.name
                    ),
                    "Assign an unused integer backing value. For an implicit member, change the preceding explicit value or assign this member an unused explicit value.",
                    member.span,
                );
            } else {
                seen_values.insert(backing, member.name.as_str());
            }
            values.push(Some(backing));
            next_value = backing.checked_add(1);
        }
        values
    }

    /// Register only `Type.Variant`; retain the short spelling solely for repair hints.
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
        if !self.scopes.define_in_root(&qualified, symbol) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate enum member `{qualified}`"),
                "Each enum member name must be unique in the program.",
                span,
            );
            return;
        }

        self.imported_candidates
            .entry(canonical_symbol_name(variant_name))
            .or_default()
            .push(qualified);
    }
}
