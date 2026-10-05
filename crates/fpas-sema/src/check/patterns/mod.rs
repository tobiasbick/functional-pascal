//! Shared typing and binding rules for recursive case patterns.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

mod coverage;
mod values;
mod variants;

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::Pattern;
use std::collections::{HashMap, HashSet};

/// Resolved variant discriminator for a recursive pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternVariant {
    /// Index within the scrutinee's nominal enum declaration.
    Enum(usize),
    /// Present option.
    Some,
    /// Empty option.
    None,
    /// Successful result.
    Ok,
    /// Failed result.
    Error,
}

/// Concrete matched type and optional variant discriminator for one AST pattern.
#[derive(Debug, Clone)]
pub struct PatternInfo {
    /// Concrete type, including generic substitutions.
    pub ty: Ty,
    /// Resolved discriminator when this pattern names a variant.
    pub variant: Option<PatternVariant>,
}

/// Pattern metadata keyed by immutable AST node identity.
pub type PatternInfoMap = HashMap<usize, PatternInfo>;

impl Checker {
    /// Check one recursive pattern and collect its immutable arm-local bindings.
    pub(in crate::check) fn check_pattern(
        &mut self,
        pattern: &Pattern,
        expected: &Ty,
    ) -> Vec<(String, Ty)> {
        let mut bindings = Vec::new();
        self.check_pattern_inner(pattern, expected, &mut bindings);
        let mut names = HashSet::new();
        bindings.retain(|(name, _)| {
            if names.insert(canonical_symbol_name(name)) {
                return true;
            }
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Pattern binding `{name}` is declared more than once"),
                "Use distinct names for bindings in one arm.",
                pattern.span(),
            );
            false
        });
        bindings
    }

    fn check_pattern_inner(
        &mut self,
        pattern: &Pattern,
        expected: &Ty,
        bindings: &mut Vec<(String, Ty)>,
    ) {
        let ty = self.resolve_visible_type(expected);
        self.pattern_infos.insert(
            crate::pattern_lookup_key(pattern),
            PatternInfo {
                ty: ty.clone(),
                variant: None,
            },
        );
        match pattern {
            Pattern::Binding { name, span } => {
                if name == "_"
                    || self
                        .import_aliases
                        .contains_key(&canonical_symbol_name(name))
                {
                    self.error_with_code(
                        SEMA_DUPLICATE_DECLARATION,
                        format!(
                            "Pattern binding `{name}` cannot shadow an import qualifier or bind `_`"
                        ),
                        "Use another binding name, or `_` without `const` to ignore a payload.",
                        *span,
                    );
                } else {
                    bindings.push((name.clone(), ty));
                }
            }
            Pattern::Wildcard(_) => {}
            Pattern::Variant {
                designator,
                arguments,
                parenthesized,
                ..
            } => {
                if let Some((variant, payloads)) =
                    self.pattern_variant(&ty, designator, arguments.len())
                {
                    if payloads.is_empty() && *parenthesized {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            "A payloadless variant pattern has no argument list",
                            "Write the qualified variant value without parentheses.",
                            pattern.span(),
                        );
                    }
                    if let Some(info) = self
                        .pattern_infos
                        .get_mut(&crate::pattern_lookup_key(pattern))
                    {
                        info.variant = Some(variant);
                    }
                    for (argument, payload) in arguments.iter().zip(payloads) {
                        self.check_pattern_inner(argument, &payload, bindings);
                    }
                }
            }
            Pattern::Value { start, end, span } => {
                if let fpas_parser::Expr::Designator(designator) = start
                    && matches!(ty, Ty::Enum(_))
                    && end.is_none()
                    && self.pattern_name_is_variant(designator)
                    && let Some((variant, _)) = self.pattern_variant(&ty, designator, 0)
                {
                    if let Some(info) = self
                        .pattern_infos
                        .get_mut(&crate::pattern_lookup_key(pattern))
                    {
                        info.variant = Some(variant);
                    }
                    return;
                }
                if let fpas_parser::Expr::Designator(designator) = start
                    && end.is_none()
                    && let [fpas_parser::DesignatorPart::Ident(name, _)] =
                        designator.parts.as_slice()
                    && self.scopes.lookup(name).is_none()
                {
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!("Pattern name `{name}` is not a known constant"),
                        format!(
                            "A plain identifier never introduces a binding. Write `const {name}` to bind the matched value, or `_` to ignore it."
                        ),
                        *span,
                    );
                    return;
                }
                let value_ty = self.check_expr_with_expected(start, &ty);
                self.check_type_compat(&ty, &value_ty, "pattern value", *span);
                self.require_static_pattern_value(start, &value_ty, false);
                if !self.supports_equality(&ty) && !ty.is_error() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Pattern value does not support equality",
                        "Use `const Name` or `_` for a payload without data equality.",
                        *span,
                    );
                }
                if let Some(end) = end {
                    let end_ty = self.check_expr(end);
                    self.check_type_compat(&ty, &end_ty, "pattern range endpoint", *span);
                    self.require_static_pattern_value(end, &end_ty, true);
                    if !matches!(ty, Ty::Integer | Ty::String) {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            "Pattern ranges require integer or string values",
                            "Match other payload types with individual values or nested variants.",
                            *span,
                        );
                    }
                }
            }
        }
    }
}

impl Checker {
    fn pattern_name_is_variant(&mut self, designator: &fpas_parser::Designator) -> bool {
        let name = self.resolve_designator_name(designator);
        self.scopes.lookup(&name).is_some_and(|symbol| {
            matches!(
                symbol.kind,
                crate::scope::SymbolKind::EnumMember
                    | crate::scope::SymbolKind::EnumVariantConstructor
            )
        })
    }
}
