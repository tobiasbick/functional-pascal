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
            .map(|field| (field.name.clone(), field.default_value.clone()))
            .collect();
        if defaults.iter().any(|(_, value)| value.is_some()) {
            self.record_defaults
                .insert(definition.name.clone(), defaults);
        }
    }

    /// Check original default expressions with only preceding values in scope.
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
            let Some(expression) = &field.default_value else {
                continue;
            };
            let Some((_, expected)) = shape
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
            else {
                continue;
            };
            let actual = self.check_expr_with_expected_record_literals(expression, expected);
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
                self.discard_info(expression).value,
            );
        }
    }
}
