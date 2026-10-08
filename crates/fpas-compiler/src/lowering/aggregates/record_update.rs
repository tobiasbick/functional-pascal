//! Record copy-update lowering.
//!
//! **Documentation:** `docs/pascal/language/types/record-update.md`

use fpas_ir::{Operation, ValueId};
use fpas_parser::{Expr, FieldInit};

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    /// Preserves the base record and evaluated fields until the update.
    pub(in crate::lowering) fn lower_record_update(
        &mut self,
        base: &Expr,
        fields: &[FieldInit],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let layout = self
            .record_layout_id(ty)
            .ok_or_else(|| unsupported(expression.span(), "record update layout"))?;
        let record = self.lower_expression(base)?;
        let record = self.save_value(record);
        let fields = fields
            .iter()
            .map(|field| {
                let (id, field_ty) = self
                    .record_field(layout, &field.name)
                    .ok_or_else(|| unsupported(field.span, "record update field"))?;
                // Overrides take the field type as their expected type,
                // so context-typed values such as `[]` get the field's element type.
                let value = self.lower_expression_as(&field.value, field_ty)?;
                Ok((id, self.save_value(value)))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        let record = self.restore_value(record, expression.span())?;
        let fields = fields
            .into_iter()
            .map(|(id, value)| Ok((id, self.restore_value(value, expression.span())?)))
            .collect::<Result<Vec<_>, CompileError>>()?;
        self.emit_value(
            Operation::UpdateRecord {
                record,
                layout,
                fields,
            },
            ty,
            expression.span(),
        )
    }
}
