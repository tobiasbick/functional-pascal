//! `var` call arguments: references to caller variables, record fields, and array elements.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use fpas_ir::{IrType, Operation, ValueId};
use fpas_parser::{Designator, DesignatorPart};

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    /// Reads the physical reference type, then specializes an erased generic collection field.
    pub(super) fn read_collection_reference(
        &mut self,
        reference: ValueId,
        requested: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<(ValueId, fpas_ir::TypeId), CompileError> {
        let reference_ty = self
            .lowered_value_type(reference)
            .ok_or_else(|| unsupported(span, "receiver reference type"))?;
        let Some(IrType::Reference(storage)) = self.type_kind(reference_ty) else {
            return Err(unsupported(span, "receiver storage type"));
        };
        let value = self.emit_value(Operation::ReferenceRead(reference), storage, span)?;
        Ok((self.value_as(value, requested, span)?, storage))
    }

    /// Lowers `var Designator` to a reference; the root and indices are evaluated once, now.
    pub(in crate::lowering) fn lower_var_argument(
        &mut self,
        designator: &Designator,
    ) -> Result<ValueId, CompileError> {
        self.lower_var_argument_prefix(designator, designator.parts.len())
    }

    /// Borrow the checked index expressions when an operation follows the storage path.
    pub(super) fn lower_var_argument_prefix(
        &mut self,
        designator: &Designator,
        part_count: usize,
    ) -> Result<ValueId, CompileError> {
        let span = designator.span;
        let (root_name, root_parts) = self.designator_root(designator)?;
        let mut ty = self
            .root_type(&root_name)
            .ok_or_else(|| unsupported(span, "`var` argument root"))?;
        let mut reference = self.reference_to_variable(&root_name, ty, span)?;
        for part in &designator.parts[root_parts..part_count] {
            match part {
                DesignatorPart::Ident(name, part_span) => {
                    let Some(IrType::Record(layout)) = self.type_kind(ty) else {
                        return Err(unsupported(
                            *part_span,
                            "`var` field argument on non-record",
                        ));
                    };
                    let (field, field_ty) = self
                        .record_field(layout, name)
                        .ok_or_else(|| unsupported(*part_span, "`var` argument field"))?;
                    let reference_ty = self.type_table.reference_type(field_ty, *part_span)?;
                    reference = self.emit_value(
                        Operation::ReferenceField {
                            reference,
                            layout,
                            field,
                        },
                        reference_ty,
                        *part_span,
                    )?;
                    ty = field_ty;
                }
                DesignatorPart::Index(index, part_span) => {
                    let Some(IrType::Array(element)) = self.type_kind(ty) else {
                        return Err(unsupported(
                            *part_span,
                            "`var` element argument on non-array",
                        ));
                    };
                    let saved = self.save_value(reference);
                    let index = self.lower_expression(index)?;
                    let collection = self.restore_value(saved, *part_span)?;
                    let reference_ty = self.type_table.reference_type(element, *part_span)?;
                    reference = self.emit_value(
                        Operation::ReferenceElement {
                            reference: collection,
                            index,
                        },
                        reference_ty,
                        *part_span,
                    )?;
                    ty = element;
                }
            }
            let part_span = match part {
                DesignatorPart::Ident(_, span) | DesignatorPart::Index(_, span) => *span,
            };
            let specialized = self.path_result_type(part_span, ty)?;
            if specialized != ty {
                // Retag the reference itself; preserve its path to caller storage.
                let reference_ty = self.type_table.reference_type(specialized, part_span)?;
                reference = self.value_as(reference, reference_ty, part_span)?;
                ty = specialized;
            }
        }
        Ok(reference)
    }
}
