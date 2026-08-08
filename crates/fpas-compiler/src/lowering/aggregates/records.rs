//! Record literal, default-field, and copy-update lowering.

use std::collections::HashMap;

use fpas_ir::{Operation, TypeId, ValueId};
use fpas_parser::{Expr, FieldInit};
use fpas_sema::Ty;

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(in crate::lowering) fn lower_record_literal(
        &mut self,
        fields: &[FieldInit],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let Ty::Record(record) = self.expression_type(expression)? else {
            return Err(unsupported(expression.span(), "record literal type"));
        };
        let ty = self.expression_ir_type(expression)?;
        let layout = self
            .record_layout_id(ty)
            .ok_or_else(|| unsupported(expression.span(), "record layout"))?;
        let provided = fields
            .iter()
            .map(|field| (field.name.to_ascii_lowercase(), &field.value))
            .collect::<HashMap<_, _>>();
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
        let mut staged = Vec::with_capacity(defaults.len());
        for (name, default) in defaults {
            let expression = provided
                .get(&name.to_ascii_lowercase())
                .copied()
                .or(default.as_ref())
                .ok_or_else(|| unsupported(expression.span(), "missing record field"))?;
            let expected = field_types
                .iter()
                .find(|(field, _)| field.eq_ignore_ascii_case(&name))
                .map(|(_, ty)| *ty)
                .ok_or_else(|| unsupported(expression.span(), "record field type"))?;
            let value = self.lower_expression_as(expression, expected)?;
            let local = self.declare_hidden_local(expected, expression.span())?;
            self.write_local(local, value, expression.span())?;
            staged.push((local, expected, expression.span()));
        }
        let values = staged
            .into_iter()
            .map(|(local, ty, span)| self.emit_value(Operation::ReadLocal(local), ty, span))
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(
            Operation::MakeRecord {
                layout,
                fields: values,
            },
            ty,
            expression.span(),
        )
    }

    pub(in crate::lowering) fn lower_record_literal_as(
        &mut self,
        fields: &[FieldInit],
        ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let layout = self
            .record_layout_id(ty)
            .ok_or_else(|| unsupported(span, "expected record layout"))?;
        let provided = fields
            .iter()
            .map(|field| (field.name.to_ascii_lowercase(), &field.value))
            .collect::<HashMap<_, _>>();
        let layout_name = self
            .record_layout_name(layout)
            .ok_or_else(|| unsupported(span, "expected record layout name"))?;
        let defaults = self
            .record_defaults
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(layout_name))
            .or_else(|| {
                self.record_defaults
                    .iter()
                    .find(|(name, _)| super::super::type_names::matches(name, layout_name))
            })
            .map(|(_, fields)| fields.clone())
            .unwrap_or_default();
        let fields = self
            .record_fields(layout)
            .ok_or_else(|| unsupported(span, "expected record fields"))?;
        let mut staged = Vec::with_capacity(fields.len());
        for (name, field_ty) in fields {
            let expression = provided
                .get(&name.to_ascii_lowercase())
                .copied()
                .or_else(|| {
                    defaults
                        .iter()
                        .find(|(field, _)| field.eq_ignore_ascii_case(&name))
                        .and_then(|(_, value)| value.as_ref())
                })
                .ok_or_else(|| unsupported(span, "missing record field"))?;
            let value = self.lower_expression_as(expression, field_ty)?;
            let local = self.declare_hidden_local(field_ty, expression.span())?;
            self.write_local(local, value, expression.span())?;
            staged.push((local, field_ty, expression.span()));
        }
        let values = staged
            .into_iter()
            .map(|(local, ty, span)| self.emit_value(Operation::ReadLocal(local), ty, span))
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
        let fields = fields
            .iter()
            .map(|field| {
                let (id, _) = self
                    .record_field(layout, &field.name)
                    .ok_or_else(|| unsupported(field.span, "record update field"))?;
                Ok((id, self.lower_expression(&field.value)?))
            })
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
