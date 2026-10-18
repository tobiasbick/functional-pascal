//! Named and positional pattern fields, normalized for case coverage.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::{Checker, PatternBindings};
use crate::check::stmt::control_flow::if_case::coverage::Pat;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_ENUM_FIELD_COUNT_MISMATCH, SEMA_INVALID_NAMED_ARGUMENT};
use fpas_lexer::Span;
use fpas_parser::PatternField;

impl Checker {
    /// Checks fields in written order and returns their coverage patterns in declaration order.
    pub(super) fn check_pattern_fields(
        &mut self,
        variant: &str,
        fields: &[PatternField],
        declared: &[(String, Ty)],
        span: Span,
        mut bindings: Option<&mut PatternBindings>,
    ) -> Vec<Pat> {
        let named = fields.iter().any(|field| field.label.is_some());
        let mut slots = vec![None; declared.len()];
        if !named && fields.len() != declared.len() {
            self.error_with_code(
                SEMA_ENUM_FIELD_COUNT_MISMATCH,
                format!("Variant '{variant}' expects {} fields, but {} were supplied.", declared.len(), fields.len()),
                "Write every field explicitly. Use `_` to ignore a field or `const Name` to bind it in a case arm.",
                span,
            );
        }
        for (written, field) in fields.iter().enumerate() {
            let index = if let Some((name, label_span)) = &field.label {
                let index = declared
                    .iter()
                    .position(|(declared, _)| declared.eq_ignore_ascii_case(name));
                if index.is_none() {
                    self.error_with_code(
                        SEMA_INVALID_NAMED_ARGUMENT,
                        format!("Variant `{variant}` has no field `{name}`"),
                        format!(
                            "Expected field names: {}.",
                            declared
                                .iter()
                                .map(|(name, _)| name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        *label_span,
                    );
                }
                index
            } else {
                Some(written).filter(|index| *index < declared.len())
            };
            let Some(index) = index else { continue };
            if let Some((_, label_span)) = &field.label {
                self.selection_types.insert(
                    (label_span.source_id, label_span.offset),
                    (*label_span, declared[index].1.clone()),
                );
            }
            if slots[index].is_some() {
                self.error_with_code(
                    SEMA_INVALID_NAMED_ARGUMENT,
                    format!(
                        "Field `{}` of `{variant}` is named more than once",
                        declared[index].0
                    ),
                    "Match each field exactly once, ignoring unused values with `_`.",
                    field
                        .label
                        .as_ref()
                        .map_or(field.pattern.span(), |(_, span)| *span),
                );
            }
            slots[index] = Some(self.check_pattern(
                &declared[index].1,
                &field.pattern,
                bindings.as_deref_mut(),
            ));
        }
        if named {
            let missing = declared
                .iter()
                .zip(&slots)
                .filter(|(_, slot)| slot.is_none())
                .map(|((name, _), _)| name.as_str())
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                self.error_with_code(
                    SEMA_INVALID_NAMED_ARGUMENT,
                    format!(
                        "Pattern `{variant}` is missing fields: {}",
                        missing.join(", ")
                    ),
                    format!("Match every field explicitly, for example `{variant}({})` to ignore its payloads.", declared.iter().map(|(name, _)| format!("{name} := _")).collect::<Vec<_>>().join(", ")),
                    span,
                );
            }
        }
        slots
            .into_iter()
            .map(|slot| slot.unwrap_or(Pat::Wild))
            .collect()
    }
}
