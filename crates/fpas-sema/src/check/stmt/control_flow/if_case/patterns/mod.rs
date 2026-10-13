//! Recursive patterns: explicit bindings, `_`, comparisons, and nested variants.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::Checker;
use super::coverage::Pat;
mod constructors;
mod values;

use crate::scope::canonical_symbol_name;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_ENUM_FIELD_COUNT_MISMATCH, SEMA_IMPLICIT_PATTERN_BINDING,
    SEMA_IS_PATTERN_BINDING, SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED, SEMA_TYPE_MISMATCH,
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
                payload,
                span,
            } => {
                if !self.destructure_matches(expected_ty, *variant, *span) {
                    return Pat::Wild;
                }
                let key = destructure_key(*variant).to_string();
                let Some(payload) = payload else {
                    return Pat::Ctor(key, Vec::new());
                };
                let payload_ty = super::labels::binding_type_for_variant(expected_ty, variant);
                let inner = self.check_pattern(&payload_ty, payload, bindings);
                Pat::Ctor(key, vec![inner])
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
        mut bindings: Option<&mut PatternBindings>,
    ) -> Pat {
        let Some((variant_name, field_types)) = self.variant_field_types(expected_ty, constructor)
        else {
            return Pat::Wild;
        };
        if fields.len() != field_types.len() {
            self.error_with_code(
                SEMA_ENUM_FIELD_COUNT_MISMATCH,
                format!(
                    "Variant '{variant_name}' expects {} field{}, but {} {} supplied.",
                    field_types.len(),
                    if field_types.len() == 1 { "" } else { "s" },
                    fields.len(),
                    if fields.len() == 1 { "was" } else { "were" },
                ),
                format!(
                    "Write {} pattern{} such as `_` or a literal to match all fields of '{variant_name}'. Bind payloads with `const Name` only in case arms.",
                    field_types.len(),
                    if field_types.len() == 1 { "" } else { "s" },
                ),
                span,
            );
            return Pat::Wild;
        }
        if let Some((_, name_span)) = fields.iter().find_map(|field| field.label.as_ref()) {
            self.error_with_code(
                SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED,
                "Enum patterns match variant fields by position",
                "Write one pattern per field in declaration order, for example `when Shape.Circle(const R):`; binding names need not match field names.",
                *name_span,
            );
        }
        let args = fields
            .iter()
            .zip(&field_types)
            .map(|(field, field_ty)| {
                self.check_pattern(field_ty, &field.pattern, bindings.as_deref_mut())
            })
            .collect();
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
