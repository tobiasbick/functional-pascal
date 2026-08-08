//! P5 collection, record, Result, Option, and positional designator lowering.

use std::collections::HashMap;

use fpas_ir::{IrType, Operation, TypeId, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr, FieldInit, PostfixOperation};
use fpas_sema::Ty;

use crate::CompileError;

use super::context::target;
use super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(super) fn lower_designator_write(
        &mut self,
        designator: &Designator,
        replacement: ValueId,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let Some(DesignatorPart::Ident(name, _)) = designator.parts.first() else {
            return Err(unsupported(designator.span, "assignment root"));
        };
        if designator.parts.len() == 1 {
            return if self.has_binding(name) {
                self.write_named_local(name, replacement, span)
            } else {
                self.write_global(name, replacement, span)
            };
        }
        let ty = self
            .root_type(name)
            .ok_or_else(|| unsupported(designator.span, "assignment root type"))?;
        let root = if self.has_binding(name) {
            self.read_named_local(name, span)?
        } else {
            self.read_global(name, span)?
        };
        let updated =
            self.lower_path_update(root, ty, &designator.parts[1..], replacement, span)?;
        if self.has_binding(name) {
            self.write_named_local(name, updated, span)
        } else {
            self.write_global(name, updated, span)
        }
    }

    fn lower_path_update(
        &mut self,
        aggregate: ValueId,
        ty: TypeId,
        parts: &[DesignatorPart],
        replacement: ValueId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let Some((part, tail)) = parts.split_first() else {
            return Ok(replacement);
        };
        match part {
            DesignatorPart::Ident(name, part_span) => {
                let IrType::Record(layout) = self
                    .type_kind(ty)
                    .ok_or_else(|| unsupported(*part_span, "assignment aggregate type"))?
                else {
                    return Err(unsupported(*part_span, "field assignment on non-record"));
                };
                let (field, field_ty) = self
                    .record_field(layout, name)
                    .ok_or_else(|| unsupported(*part_span, "assignment field"))?;
                let value = if tail.is_empty() {
                    replacement
                } else {
                    let child = self.emit_value(
                        Operation::LoadField {
                            record: aggregate,
                            layout,
                            field,
                        },
                        field_ty,
                        *part_span,
                    )?;
                    self.lower_path_update(child, field_ty, tail, replacement, span)?
                };
                self.emit_value(
                    Operation::UpdateRecord {
                        record: aggregate,
                        layout,
                        fields: vec![(field, value)],
                    },
                    ty,
                    span,
                )
            }
            DesignatorPart::Index(index, part_span) => {
                let element_ty = match self.type_kind(ty) {
                    Some(IrType::Array(element)) => element,
                    Some(IrType::Dictionary { value, .. }) => value,
                    _ => {
                        return Err(unsupported(
                            *part_span,
                            "index assignment on non-collection",
                        ));
                    }
                };
                let index = self.lower_expression(index)?;
                let value = if tail.is_empty() {
                    replacement
                } else {
                    let child = self.emit_value(
                        Operation::IndexGet {
                            collection: aggregate,
                            index,
                        },
                        element_ty,
                        *part_span,
                    )?;
                    self.lower_path_update(child, element_ty, tail, replacement, span)?
                };
                self.emit_value(
                    Operation::IndexSet {
                        collection: aggregate,
                        index,
                        value,
                    },
                    ty,
                    span,
                )
            }
        }
    }

    pub(super) fn lower_designator_read(
        &mut self,
        designator: &Designator,
    ) -> Result<ValueId, CompileError> {
        let Some(DesignatorPart::Ident(name, _)) = designator.parts.first() else {
            return Err(unsupported(designator.span, "designator root"));
        };
        let mut ty = self
            .root_type(name)
            .ok_or_else(|| unsupported(designator.span, "unresolved designator"))?;
        let mut value = if self.has_binding(name) {
            self.read_named_local(name, designator.span)?
        } else {
            self.read_global(name, designator.span)?
        };
        for part in &designator.parts[1..] {
            (value, ty) = self.lower_designator_part(value, ty, part)?;
        }
        Ok(value)
    }

    pub(super) fn lower_postfix(
        &mut self,
        base: &Expr,
        operations: &[PostfixOperation],
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let mut value = self.lower_expression(base)?;
        let mut ty = self.expression_ir_type(base)?;
        for operation in operations {
            if let Some((member_value, member_ty)) = self.lower_postfix_member(value, operation)? {
                value = member_value;
                ty = member_ty;
                continue;
            }
            match operation {
                PostfixOperation::Field { name, span } => {
                    let part = DesignatorPart::Ident(name.clone(), *span);
                    (value, ty) = self.lower_designator_part(value, ty, &part)?;
                }
                PostfixOperation::Index { index, span } => {
                    let part = DesignatorPart::Index((**index).clone(), *span);
                    (value, ty) = self.lower_designator_part(value, ty, &part)?;
                }
                PostfixOperation::MethodCall { .. } => {
                    return Err(unsupported(span, "postfix method call"));
                }
            }
        }
        Ok(value)
    }

    pub(super) fn lower_designator_part(
        &mut self,
        value: ValueId,
        ty: TypeId,
        part: &DesignatorPart,
    ) -> Result<(ValueId, TypeId), CompileError> {
        match part {
            DesignatorPart::Ident(name, span) => {
                let IrType::Record(layout) = self
                    .type_kind(ty)
                    .ok_or_else(|| unsupported(*span, "unknown aggregate type"))?
                else {
                    return Err(unsupported(*span, "field access on non-record"));
                };
                let (field, field_ty) = self
                    .record_field(layout, name)
                    .ok_or_else(|| unsupported(*span, "unknown record field"))?;
                let result = self.emit_value(
                    Operation::LoadField {
                        record: value,
                        layout,
                        field,
                    },
                    field_ty,
                    *span,
                )?;
                Ok((result, field_ty))
            }
            DesignatorPart::Index(index, span) => {
                let result_ty = match self.type_kind(ty) {
                    Some(IrType::Array(element)) => element,
                    Some(IrType::Dictionary { value, .. }) => value,
                    Some(IrType::String) => super::types::STRING,
                    _ => return Err(unsupported(*span, "indexing non-collection")),
                };
                let index = self.lower_expression(index)?;
                let result = self.emit_value(
                    Operation::IndexGet {
                        collection: value,
                        index,
                    },
                    result_ty,
                    *span,
                )?;
                Ok((result, result_ty))
            }
        }
    }

    pub(super) fn lower_array_literal(
        &mut self,
        values: &[Expr],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let values = values
            .iter()
            .map(|value| self.lower_expression(value))
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(Operation::MakeArray(values), ty, expression.span())
    }

    pub(super) fn lower_dictionary_literal(
        &mut self,
        pairs: &[(Expr, Expr)],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let pairs = pairs
            .iter()
            .map(|(key, value)| Ok((self.lower_expression(key)?, self.lower_expression(value)?)))
            .collect::<Result<Vec<_>, CompileError>>()?;
        self.emit_value(Operation::MakeDictionary(pairs), ty, expression.span())
    }

    pub(super) fn lower_record_literal(
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
        let mut values = Vec::with_capacity(defaults.len());
        for (name, default) in defaults {
            let expression = provided
                .get(&name.to_ascii_lowercase())
                .copied()
                .or(default.as_ref())
                .ok_or_else(|| unsupported(expression.span(), "missing record field"))?;
            values.push(self.lower_expression(expression)?);
        }
        self.emit_value(
            Operation::MakeRecord {
                layout,
                fields: values,
            },
            ty,
            expression.span(),
        )
    }

    pub(super) fn lower_record_update(
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

    pub(super) fn lower_wrapper(
        &mut self,
        operand: Option<&Expr>,
        expression: &Expr,
        kind: u8,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let operation = match (kind, operand) {
            (0, Some(value)) => Operation::MakeOk(self.lower_expression(value)?),
            (1, Some(value)) => Operation::MakeError(self.lower_expression(value)?),
            (2, Some(value)) => Operation::MakeSome(self.lower_expression(value)?),
            (3, None) => Operation::MakeNone,
            _ => return Err(unsupported(expression.span(), "wrapper construction")),
        };
        self.emit_value(operation, ty, expression.span())
    }

    pub(super) fn lower_try(
        &mut self,
        inner: &Expr,
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let wrapper_ty = self.expression_ir_type(inner)?;
        let result_ty = self.expression_ir_type(expression)?;
        let wrapper = self.lower_expression(inner)?;
        let wrapper_local = self.declare_hidden_local(wrapper_ty, expression.span())?;
        self.write_local(wrapper_local, wrapper, expression.span())?;
        let test = match self.type_kind(wrapper_ty) {
            Some(IrType::Result { .. }) => Operation::IsResultOk(wrapper),
            Some(IrType::Option(_)) => Operation::IsOptionSome(wrapper),
            _ => return Err(unsupported(expression.span(), "try operand")),
        };
        let condition = self.emit_value(test, super::types::BOOLEAN, expression.span())?;
        let success = self.new_block(expression.span())?;
        let failure = self.new_block(expression.span())?;
        let merge = self.new_block(expression.span())?;
        self.terminate(fpas_ir::Terminator::Branch {
            condition,
            then_target: target(success),
            else_target: target(failure),
        })?;
        self.switch_to(failure);
        let failure_value = self.emit_value(
            Operation::ReadLocal(wrapper_local),
            wrapper_ty,
            expression.span(),
        )?;
        self.terminate(fpas_ir::Terminator::Return(Some(failure_value)))?;
        self.switch_to(success);
        let success_wrapper = self.emit_value(
            Operation::ReadLocal(wrapper_local),
            wrapper_ty,
            expression.span(),
        )?;
        let operation = match self.type_kind(wrapper_ty) {
            Some(IrType::Result { .. }) => Operation::UnwrapOk(success_wrapper),
            Some(IrType::Option(_)) => Operation::UnwrapSome(success_wrapper),
            _ => return Err(unsupported(expression.span(), "try operand")),
        };
        let payload = self.emit_value(operation, result_ty, expression.span())?;
        let payload_local = self.declare_hidden_local(result_ty, expression.span())?;
        self.write_local(payload_local, payload, expression.span())?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(
            Operation::ReadLocal(payload_local),
            result_ty,
            expression.span(),
        )
    }
}
