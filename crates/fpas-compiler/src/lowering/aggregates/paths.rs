//! Nested designator reads, writes, and postfix field or index access.

use crate::CompileError;
use fpas_ir::{IrType, Operation, TypeId, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(in crate::lowering) fn lower_designator_write(
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

    pub(in crate::lowering) fn lower_designator_read(
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

    pub(in crate::lowering) fn lower_postfix(
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

    pub(in crate::lowering) fn lower_designator_part(
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
                    Some(IrType::String) => super::super::types::STRING,
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
}
