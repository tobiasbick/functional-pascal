//! Named record construction, default-field, and copy-update lowering.

use std::collections::HashMap;

use fpas_ir::{Operation, TypeId, ValueId};
use fpas_parser::{Expr, FieldInit};
use fpas_sema::Ty;

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(in crate::lowering) fn lower_record_construction(
        &mut self,
        fields: &[FieldInit],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let Ty::Record(record) = self.expression_type(expression)? else {
            return Err(unsupported(expression.span(), "record construction type"));
        };
        let ty = self.expression_ir_type(expression)?;
        let layout = self
            .record_layout_id(ty)
            .ok_or_else(|| unsupported(expression.span(), "record layout"))?;
        let defaults = self
            .record_defaults
            .get(&record.name)
            .cloned()
            .unwrap_or_else(|| {
                record
                    .fields
                    .iter()
                    .map(|(name, _)| (name.clone(), None))
                    .collect()
            });
        let field_types = self
            .record_fields(layout)
            .ok_or_else(|| unsupported(expression.span(), "record fields"))?;
        self.lower_record_fields_in_order(
            layout,
            ty,
            expression.span(),
            fields,
            &field_types,
            &defaults,
        )
    }

    fn lower_record_fields_in_order(
        &mut self,
        layout: fpas_ir::RecordLayoutId,
        ty: TypeId,
        span: fpas_lexer::Span,
        supplied: &[FieldInit],
        declared: &[(String, TypeId)],
        defaults: &[(String, Option<std::sync::Arc<Expr>>)],
    ) -> Result<ValueId, CompileError> {
        let mut staged = HashMap::new();
        for field in supplied {
            let field_ty = declared
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                .map(|(_, ty)| *ty)
                .ok_or_else(|| unsupported(field.span, "record field type"))?;
            let expression = &field.value;
            let value = self.lower_expression_as(expression, field_ty)?;
            let local = self.declare_hidden_local(field_ty, expression.span())?;
            self.write_local(local, value, expression.span())?;
            staged.insert(
                field.name.to_ascii_lowercase(),
                (local, field_ty, expression.span()),
            );
        }
        for (name, field_ty) in declared {
            let key = name.to_ascii_lowercase();
            if staged.contains_key(&key) {
                continue;
            }
            let expression = defaults
                .iter()
                .find(|(field, _)| field.eq_ignore_ascii_case(name))
                .and_then(|(_, expression)| expression.as_deref())
                .ok_or_else(|| unsupported(span, "missing record field"))?;
            let value = self.lower_expression_as(expression, *field_ty)?;
            let local = self.declare_hidden_local(*field_ty, expression.span())?;
            self.write_local(local, value, expression.span())?;
            staged.insert(key, (local, *field_ty, expression.span()));
        }
        let values = declared
            .iter()
            .map(|(name, _)| {
                let (local, ty, span) = staged
                    .remove(&name.to_ascii_lowercase())
                    .ok_or_else(|| unsupported(span, "staged record field"))?;
                self.emit_value(Operation::ReadLocal(local), ty, span)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(
            Operation::MakeRecord {
                layout,
                fields: values,
            },
            ty,
            span,
        )
    }

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
                // Like constructors, overrides take the field type as their expected type,
                // so context-typed values such as `[]` get the field's element type.
                let value = self.lower_expression_as(&field.value, field_ty)?;
                let value = self.coerce_value_type(value, field_ty, field.span)?;
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
