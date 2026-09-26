//! Property assignment checking (`Button.Text := …`).
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use super::Checker;
use super::assignment::MemberAssignmentTarget;
use crate::types::{PropertyTy, RecordTy};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

use super::super::context::PropertyWriteInfo;

impl Checker {
    /// Type-checks an assignment to a record property and records its setter call.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(super) fn check_property_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: Span,
        member: MemberAssignmentTarget<'_>,
        property: PropertyTy,
    ) {
        let property_name = member.name;
        let Some(setter) = property.setter.clone() else {
            let record_name = &member.record_ty.name;
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Property `{record_name}.{property_name}` is read-only"),
                format!(
                    "Read `{property_name}` instead, or add a `write` accessor to the property."
                ),
                member.span,
            );
            let _ = self.check_expr(value);
            return;
        };

        let value_ty = self.check_expr_with_expected_record_literals(value, &property.ty);
        self.check_type_compat(&property.ty, &value_ty, "property assignment", span);

        let key = crate::designator_lookup_key(target);
        self.property_writes.insert(
            key,
            PropertyWriteInfo {
                setter_name: setter,
                receiver_part_count: target.parts.len() - 1,
                receiver_reads: member.receiver_reads,
            },
        );
    }

    /// Finds the property `member` on `record_ty` or its canonical declaration.
    pub(super) fn find_record_property_on_type(
        &self,
        record_ty: &RecordTy,
        member: &str,
    ) -> Option<PropertyTy> {
        self.find_record_member_on_type(record_ty, member, |record| &record.properties)
    }
}
