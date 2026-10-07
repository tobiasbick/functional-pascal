//! `var` call arguments: references to caller variables, record fields, and array elements.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use fpas_ir::{IrType, Operation, ValueId};
use fpas_parser::{Designator, DesignatorPart};

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    /// Lowers `var Designator` to a reference; the root and indices are evaluated once, now.
    pub(in crate::lowering) fn lower_var_argument(
        &mut self,
        designator: &Designator,
    ) -> Result<ValueId, CompileError> {
        let span = designator.span;
        let (root_name, root_parts) = self.designator_root(designator)?;
        let mut ty = self
            .root_type(&root_name)
            .ok_or_else(|| unsupported(span, "`var` argument root"))?;
        let mut reference = self.reference_to_variable(&root_name, ty, span)?;
        for part in &designator.parts[root_parts..] {
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
        }
        Ok(reference)
    }
}
