//! Full symbol resolution for nested enum, Option, and Result patterns.

use super::{Checker, PatternVariant};
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_ENUM_FIELD_COUNT_MISMATCH, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};
use fpas_parser::{Designator, DesignatorPart};

impl Checker {
    /// Resolve a qualified variant against the expected nominal owner and payload types.
    pub(super) fn pattern_variant(
        &mut self,
        expected: &Ty,
        designator: &Designator,
        count: usize,
    ) -> Option<(PatternVariant, Vec<Ty>)> {
        let name = Self::designator_name(designator);
        let span = designator.span;
        let parts = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.to_ascii_lowercase()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        if parts.len() < 2 {
            self.error_with_code(SEMA_TYPE_MISMATCH, "Variant patterns require their declaring type qualifier",
                "Write `Choice.Present(const Value)`, `Option.Some(const Value)`, or `Result.Ok(const Value)`.", span);
            return None;
        }
        let builtin = match (parts.as_slice(), expected) {
            ([owner, variant], Ty::Option(inner)) if owner == "option" => match variant.as_str() {
                "some" => Some((PatternVariant::Some, vec![(**inner).clone()])),
                "none" => Some((PatternVariant::None, vec![])),
                _ => None,
            },
            ([owner, variant], Ty::Result(ok, error)) if owner == "result" => {
                match variant.as_str() {
                    "ok" => Some((PatternVariant::Ok, vec![(**ok).clone()])),
                    "error" => Some((PatternVariant::Error, vec![(**error).clone()])),
                    _ => None,
                }
            }
            _ => None,
        };
        let (variant, payloads) = if let Some(builtin) = builtin {
            builtin
        } else {
            let resolved = self.resolve_source_name(&name, span);
            self.ensure_fq_std_unit_loaded(&resolved);
            let Some(symbol) = self.scopes.lookup(&resolved).cloned() else {
                self.error_with_code(
                    SEMA_UNKNOWN_NAME,
                    format!("Unknown pattern variant `{name}`"),
                    "Use a qualified variant from the matched type.",
                    span,
                );
                return None;
            };
            let actual = self.resolve_visible_type(&symbol.ty);
            let belongs = matches!((&actual, expected), (Ty::Enum(a), Ty::Enum(e))
                if a.name.eq_ignore_ascii_case(&e.name)
                    && (a.type_args.is_empty() || actual.assignment_compatible_with(expected)));
            if !belongs
                || !matches!(
                    symbol.kind,
                    SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor
                )
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Pattern variant `{name}` does not belong to `{expected}`"),
                    "Use the matched type's declaration and concrete type arguments.",
                    span,
                );
                return None;
            }
            let Ty::Enum(enumeration) = expected else {
                return None;
            };
            let index = enumeration.variants.iter().position(|variant| {
                parts
                    .last()
                    .is_some_and(|name| variant.name.eq_ignore_ascii_case(name))
            })?;
            (
                PatternVariant::Enum(index),
                enumeration.variants[index]
                    .fields
                    .iter()
                    .map(|(_, ty)| ty.clone())
                    .collect(),
            )
        };
        if payloads.len() != count {
            self.error_with_code(
                SEMA_ENUM_FIELD_COUNT_MISMATCH,
                format!(
                    "Pattern `{name}` expects {} payload patterns, found {count}",
                    payloads.len()
                ),
                "Match each payload with a nested pattern, `const Name`, or `_`.",
                span,
            );
            return None;
        }
        Some((variant, payloads))
    }
}
