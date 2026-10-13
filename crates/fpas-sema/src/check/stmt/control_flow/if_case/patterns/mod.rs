//! Recursive patterns: explicit bindings, `_`, comparisons, and nested variants.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::Checker;
use super::coverage::Pat;
mod constructors;
mod fields;
mod values;

use crate::scope::canonical_symbol_name;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_ENUM_FIELD_COUNT_MISMATCH, SEMA_IMPLICIT_PATTERN_BINDING,
    SEMA_IS_PATTERN_BINDING, SEMA_TYPE_MISMATCH,
};
use fpas_lexer::Span;
use fpas_parser::{Designator, DestructureVariant, Pattern, PatternField};

/// Names a pattern binds, with their types, in source order.
pub(super) type PatternBindings = Vec<(String, Ty)>;

impl Checker {
    /// Checks a nonbinding pattern predicate using the same value and variant rules as case.
    ///
    /// **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`
    pub(in crate::check) fn check_is_pattern(&mut self, value_ty: &Ty, pattern: &Pattern) {
        if let Pattern::Wildcard(span) = pattern {
            self.reject_whole_value_wildcard(*span);
            return;
        }
        self.check_pattern(value_ty, pattern, None);
    }

    /// Checks a complete label pattern and rejects a name bound twice inside it.
    pub(super) fn check_label_pattern(
        &mut self,
        expected_ty: &Ty,
        pattern: &Pattern,
    ) -> (PatternBindings, Pat) {
        let mut bindings = PatternBindings::new();
        let pat = self.check_pattern(expected_ty, pattern, Some(&mut bindings));
        let mut unique = PatternBindings::with_capacity(bindings.len());
        for (name, ty) in bindings {
            if unique.iter().any(|(existing, _)| {
                canonical_symbol_name(existing) == canonical_symbol_name(&name)
            }) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Pattern binding `{name}` is declared more than once"),
                    "Use a distinct binding name in each position of one pattern.",
                    pattern.span(),
                );
            } else {
                unique.push((name, ty));
            }
        }
        (unique, pat)
    }

    /// Checks one pattern against the type of the value it matches.
    fn check_pattern(
        &mut self,
        expected_ty: &Ty,
        pattern: &Pattern,
        mut bindings: Option<&mut PatternBindings>,
    ) -> Pat {
        let expected_ty = self.resolve_visible_type(expected_ty);
        let span = pattern.span();
        self.pattern_types
            .insert((span.source_id, span.offset), expected_ty.clone());
        let expected_ty = &expected_ty;
        match pattern {
            Pattern::Binding { name, .. } => {
                if let Some(bindings) = bindings {
                    bindings.push((name.clone(), expected_ty.clone()));
                } else {
                    self.error_with_code(
                        SEMA_IS_PATTERN_BINDING,
                        format!("`is` cannot bind the name `{name}`"),
                        "Use a case arm to bind data, for example `when Some(const Value): Use(Value);`. Use `Some(_)` when only testing presence.",
                        span,
                    );
                }
                Pat::Wild
            }
            Pattern::Wildcard(_) => Pat::Wild,
            Pattern::Value(expr) => self.check_value_pattern(expected_ty, expr, bindings),
            Pattern::Variant {
                constructor,
                fields,
                span,
            } => self
                .check_conversion_label(expected_ty, constructor, pattern, *span)
                .unwrap_or_else(|| {
                    if fields.is_empty()
                        && let Some(enumeration) = self.resolve_enum_ty(expected_ty)
                        && let Some(fpas_parser::DesignatorPart::Ident(name, _)) =
                            constructor.parts.last()
                        && enumeration.variants.iter().any(|variant| {
                            variant.name.eq_ignore_ascii_case(name) && variant.fields.is_empty()
                        })
                    {
                        self.error_with_code(
                            SEMA_ENUM_FIELD_COUNT_MISMATCH,
                            format!("Fieldless variant `{name}` does not take parentheses"),
                            format!("Write `{name}` without `()` to match a fieldless variant."),
                            *span,
                        );
                    }
                    self.check_variant_pattern(
                        expected_ty,
                        constructor,
                        fields,
                        *span,
                        bindings.as_deref_mut(),
                    )
                }),
            Pattern::Destructure {
                variant,
                fields,
                span,
            } => {
                if !self.destructure_matches(expected_ty, *variant, *span) {
                    return Pat::Wild;
                }
                let key = destructure_key(*variant).to_string();
                if *variant == DestructureVariant::None {
                    return Pat::Ctor(key, Vec::new());
                }
                let payload_ty = super::labels::binding_type_for_variant(expected_ty, variant);
                let declared = if *variant == DestructureVariant::None {
                    Vec::new()
                } else {
                    vec![("Value".into(), payload_ty)]
                };
                let args = self.check_pattern_fields(&key, fields, &declared, *span, bindings);
                Pat::Ctor(key, args)
            }
        }
    }

    /// Validates a variant pattern against the enum type of the matched value.
    fn check_variant_pattern(
        &mut self,
        expected_ty: &Ty,
        constructor: &Designator,
        fields: &[PatternField],
        span: Span,
        bindings: Option<&mut PatternBindings>,
    ) -> Pat {
        let Some((variant_name, field_types)) = self.variant_field_types(expected_ty, constructor)
        else {
            return Pat::Wild;
        };
        let args = self.check_pattern_fields(&variant_name, fields, &field_types, span, bindings);
        Pat::Ctor(variant_name.to_ascii_lowercase(), args)
    }

    fn implicit_binding(
        &mut self,
        name: &str,
        expected_ty: &Ty,
        span: Span,
        bindings: Option<&mut PatternBindings>,
    ) -> Pat {
        self.error_with_code(
            SEMA_IMPLICIT_PATTERN_BINDING,
            format!("Pattern binding `{name}` must be written `const {name}`"),
            format!(
                "Bind the value in a case arm with `const {name}`, use `_` to ignore a field, or name a compile-time constant to compare with."
            ),
            span,
        );
        if let Some(bindings) = bindings {
            bindings.push((name.to_string(), expected_ty.clone()));
        }
        Pat::Wild
    }

    fn destructure_matches(
        &mut self,
        expected_ty: &Ty,
        variant: DestructureVariant,
        span: Span,
    ) -> bool {
        let valid = matches!(
            (expected_ty, variant),
            (
                Ty::Result(_, _),
                DestructureVariant::Ok | DestructureVariant::Error
            ) | (
                Ty::Option(_),
                DestructureVariant::Some | DestructureVariant::None
            ) | (Ty::Error, _)
        );
        if !valid {
            let hint = match variant {
                DestructureVariant::Ok | DestructureVariant::Error => {
                    "Use `Ok(_)` and `Error(_)` with `Result`. Bind payloads with `const Name` only in case arms."
                }
                DestructureVariant::Some | DestructureVariant::None => {
                    "Use `Some(_)` and `None` with `Option`. Bind payloads with `const Name` only in case arms."
                }
            };
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Pattern `{}` does not match a value of type `{expected_ty}`",
                    destructure_key_display(variant),
                ),
                hint,
                span,
            );
        }
        valid && !expected_ty.is_error()
    }
}

fn destructure_key(variant: DestructureVariant) -> &'static str {
    match variant {
        DestructureVariant::Ok => "ok",
        DestructureVariant::Error => "error",
        DestructureVariant::Some => "some",
        DestructureVariant::None => "none",
    }
}

fn destructure_key_display(variant: DestructureVariant) -> &'static str {
    match variant {
        DestructureVariant::Ok => "Ok",
        DestructureVariant::Error => "Error",
        DestructureVariant::Some => "Some",
        DestructureVariant::None => "None",
    }
}
