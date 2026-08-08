//! Collection, Result, Option, and positional designator lowering.

mod global_index_path;
mod records;

use crate::CompileError;
use fpas_ir::{IrType, Operation, TypeId, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};

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
        if !self.has_binding(name)
            && self.lower_global_index_path_write(
                name,
                &designator.parts[1..],
                replacement,
                span,
            )?
        {
            return Ok(());
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

    pub(super) fn lower_array_literal_as(
        &mut self,
        values: &[Expr],
        ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let element_ty = match self.type_kind(ty) {
            Some(IrType::Array(element)) => element,
            _ => return Err(unsupported(span, "expected array type")),
        };
        let values = values
            .iter()
            .map(|value| self.lower_expression_as(value, element_ty))
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(Operation::MakeArray(values), ty, span)
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

    pub(super) fn lower_wrapper(
        &mut self,
        operand: Option<&Expr>,
        expression: &Expr,
        kind: u8,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        self.lower_wrapper_as(operand, kind, ty, expression.span())
    }

    pub(super) fn lower_wrapper_as(
        &mut self,
        operand: Option<&Expr>,
        kind: u8,
        ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let expected_payload = match (kind, self.type_kind(ty)) {
            (0, Some(IrType::Result { ok, .. })) => Some(ok),
            (1, Some(IrType::Result { error, .. })) => Some(error),
            (2, Some(IrType::Option(value))) => Some(value),
            _ => None,
        };
        let lower_operand = |context: &mut Self, value: &Expr| match expected_payload {
            Some(expected) => context.lower_expression_as(value, expected),
            None => context.lower_expression(value),
        };
        let operation = match (kind, operand) {
            (0, Some(value)) => Operation::MakeOk(lower_operand(self, value)?),
            (1, Some(value)) => Operation::MakeError(lower_operand(self, value)?),
            (2, Some(value)) => Operation::MakeSome(lower_operand(self, value)?),
            (3, None) => Operation::MakeNone,
            _ => return Err(unsupported(span, "wrapper construction")),
        };
        self.emit_value(operation, ty, span)
    }

    pub(super) fn lower_expression_as(
        &mut self,
        expression: &Expr,
        expected: TypeId,
    ) -> Result<ValueId, CompileError> {
        match expression {
            Expr::RecordLiteral { fields, span } => {
                self.lower_record_literal_as(fields, expected, *span)
            }
            Expr::ArrayLiteral(values, span) => {
                self.lower_array_literal_as(values, expected, *span)
            }
            Expr::ResultOk(value, span) => self.lower_wrapper_as(Some(value), 0, expected, *span),
            Expr::ResultError(value, span) => {
                self.lower_wrapper_as(Some(value), 1, expected, *span)
            }
            Expr::OptionSome(value, span) => self.lower_wrapper_as(Some(value), 2, expected, *span),
            Expr::OptionNone(span) => self.lower_wrapper_as(None, 3, expected, *span),
            _ => self.lower_expression(expression),
        }
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
        let function_result = self.current_result_type();
        let propagated = match self.type_kind(wrapper_ty) {
            Some(IrType::Result { error, .. }) => {
                let error = self.emit_value(
                    Operation::UnwrapError(failure_value),
                    error,
                    expression.span(),
                )?;
                self.emit_value(
                    Operation::MakeError(error),
                    function_result,
                    expression.span(),
                )?
            }
            Some(IrType::Option(_)) => {
                self.emit_value(Operation::MakeNone, function_result, expression.span())?
            }
            _ => return Err(unsupported(expression.span(), "try operand")),
        };
        self.terminate(fpas_ir::Terminator::Return(Some(propagated)))?;
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
