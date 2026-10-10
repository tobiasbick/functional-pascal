//! Field-default registration and checking in the declaration's original scope.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::Checker;
use crate::types::Ty;
use fpas_parser::{RecordType, TypeDef};
use std::collections::HashSet;

impl Checker {
    /// Retain defaults for contextual record construction before the declaration is checked.
    pub(super) fn register_record_defaults(&mut self, definition: &TypeDef, record: &RecordType) {
        let mut seen = HashSet::new();
        let defaults: Vec<_> = record
            .fields
            .iter()
            .filter(|field| seen.insert(field.name.to_ascii_lowercase()))
            .map(|field| {
                (
                    field.name.clone(),
                    field.default_value.clone().map(std::sync::Arc::new),
                )
            })
            .collect();
        if defaults.iter().any(|(_, value)| value.is_some()) {
            self.record_defaults
                .insert(definition.name.clone(), defaults);
        }
    }

    /// Check retained defaults with stable identity and only preceding values in scope.
    pub(super) fn check_record_defaults(&mut self, definition: &TypeDef, record: &RecordType) {
        let Some(symbol) = self.scopes.lookup_type(&definition.name) else {
            return;
        };
        let Ty::Record(shape) = symbol.ty.clone() else {
            return;
        };
        let mut seen = HashSet::new();
        for field in &record.fields {
            if !seen.insert(field.name.to_ascii_lowercase()) {
                continue;
            }
            let expression = self
                .record_defaults
                .get(&definition.name)
                .and_then(|fields| {
                    fields
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                })
                .and_then(|(_, value)| value.clone());
            let Some(expression) = expression else {
                continue;
            };
            let Some((_, expected)) = shape
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
            else {
                continue;
            };
            let actual = self.check_expr_with_expected(&expression, Some(expected));
            self.check_type_compat(
                expected,
                &actual,
                &format!("default value for field `{}`", field.name),
                field.span,
            );
            self.record_default_discard.insert(
                (
                    definition.name.to_ascii_lowercase(),
                    field.name.to_ascii_lowercase(),
                ),
                self.discard_info(&expression).value,
            );
            let compile_time = self.const_initializer_is_compile_time_known(&expression, expected);
            self.record_default_constants
                .insert(Self::expr_lookup_key(&expression), compile_time);
            let value = compile_time
                .then(|| self.constant_field_value(&expression))
                .flatten();
            self.record_default_values
                .insert(Self::expr_lookup_key(&expression), value);
        }
    }
}
