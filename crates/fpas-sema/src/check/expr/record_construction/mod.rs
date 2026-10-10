//! Named construction of concrete record types using normal call-target lookup.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

mod fields;
mod inference;

use super::Checker;
use crate::types::{RecordTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::sync::Arc;

impl Checker {
    /// Validate fields without turning routine calls into record constructors.
    pub(crate) fn check_record_construction(
        &mut self,
        key: usize,
        record: &Arc<RecordTy>,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        self.record_constructions.insert(key);
        let fields: Vec<_> = args.iter().filter_map(|argument| {
            let Expr::NamedArgument { name, name_span, value, .. } = argument else {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Record construction of `{}` requires named fields", record.name),
                    "Use `TypeName(Field := Value, ...)`; positional record construction is not supported.",
                    argument.span(),
                );
                self.check_expr(argument);
                return None;
            };
            Some((name.as_str(), value.as_ref(), *name_span))
        }).collect();
        self.validate_unique_record_field_names(
            fields.iter().map(|&(name, _, span)| (name, span)),
            "record construction",
        );
        let record = self.infer_record_construction(record, &fields, expected, span);
        self.validate_typed_record_fields(&fields, &record, span);
        for (_, value, _) in &fields {
            self.prechecked_receivers
                .remove(&Self::expr_lookup_key(value));
        }
        let ty = Ty::Record(record);
        let task_free = self.record_fields_are_task_free(
            fields
                .iter()
                .map(|&(name, value, _)| (name, value))
                .collect(),
            &ty,
        );
        self.discard_exprs.insert(
            key,
            fpas_unit::interface::DiscardInfo {
                value: task_free,
                ..Default::default()
            },
        );
        ty
    }
}
