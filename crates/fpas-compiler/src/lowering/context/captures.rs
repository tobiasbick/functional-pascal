//! Capture storage forwarding by lexical declaration identity.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`

use fpas_ir::{Operation, ValueId};
use fpas_lexer::Span;

use super::{BindingStorage, CaptureInput, LoweringContext};
use crate::CompileError;
use crate::error::internal_compiler_error;

impl LoweringContext {
    /// Read capture storage at its original declaration despite caller-side shadowing.
    ///
    /// Documentation: `docs/pascal/language/functions/closures.md`
    pub(in crate::lowering) fn read_capture(
        &mut self,
        capture: &CaptureInput,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let Some(binding) = self.bindings.iter().rev().find(|binding| {
            binding.name.eq_ignore_ascii_case(&capture.name)
                && binding.declaration == capture.declaration
        }) else {
            return Err(internal_compiler_error(
                format!(
                    "Capture `{}` was not present at its declaration in register-lowering scope metadata.",
                    capture.name
                ),
                "This is an internal compiler error. Re-run compilation and report the source program.",
                span.line,
                span.column,
            ));
        };
        let BindingStorage::Local(local) = binding.storage;
        let (ty, cell, reference) = (binding.ty, binding.cell, binding.reference);
        let storage_ty = self.binding_storage_type(ty, cell, reference, span)?;
        self.emit_value(Operation::ReadLocal(local), storage_ty, span)
    }
}
