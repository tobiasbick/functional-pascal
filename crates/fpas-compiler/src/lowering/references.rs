//! Written-order reservation and selection of source caller storage.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use super::context::{LoweringContext, unsupported};
use crate::CompileError;
use fpas_ir::{IrType, Operation, ReferenceOperation, ValueId};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart};

impl LoweringContext {
    /// Reserve one root and freeze each checked field or index before later arguments.
    pub(super) fn lower_var_argument(
        &mut self,
        target: &Designator,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let (name, consumed) = self
            .designator_root(target)
            .ok_or_else(|| unsupported(span, "var storage root"))?;
        let mut ty = self
            .root_type(&name)
            .ok_or_else(|| unsupported(span, "var root type"))?;
        let root = if self.has_binding(&name) {
            self.read_local_storage_root(&name, span)?
        } else {
            self.read_global_storage_root(&name, span)?
        };
        let reference_ty = self.reference_type(ty, span)?;
        let mut reference = self.emit_value(
            Operation::Reference(ReferenceOperation::Reserve(root)),
            reference_ty,
            span,
        )?;
        for part in &target.parts[consumed..] {
            match part {
                DesignatorPart::Ident(name, part_span) => {
                    let Some(IrType::Record(layout)) = self.type_kind(ty) else {
                        return Err(unsupported(*part_span, "var field selection"));
                    };
                    let (field, field_ty) = self
                        .record_field(layout, name)
                        .ok_or_else(|| unsupported(*part_span, "var stored field"))?;
                    ty = field_ty;
                    let selected_ty = self.reference_type(ty, *part_span)?;
                    reference = self.emit_value(
                        Operation::Reference(ReferenceOperation::Field {
                            reference,
                            layout,
                            field,
                        }),
                        selected_ty,
                        *part_span,
                    )?;
                }
                DesignatorPart::Index(index, part_span) => {
                    ty = match self.type_kind(ty) {
                        Some(IrType::Array(element)) => element,
                        Some(IrType::Dictionary { value, .. }) => value,
                        _ => return Err(unsupported(*part_span, "var collection selection")),
                    };
                    let retained = self.save_value(reference);
                    let index = self.lower_expression(index)?;
                    reference = self.restore_value(retained, *part_span)?;
                    let selected_ty = self.reference_type(ty, *part_span)?;
                    reference = self.emit_value(
                        Operation::Reference(ReferenceOperation::Index { reference, index }),
                        selected_ty,
                        *part_span,
                    )?;
                }
            }
        }
        Ok(reference)
    }
}
