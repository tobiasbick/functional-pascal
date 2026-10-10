//! Typed assignment conversion between concrete and erased generic values.
//! See `docs/pascal/language/types/generics.md`.

use super::LoweringContext;
use crate::CompileError;
use fpas_ir::{Operation, TypeId, ValueId};
use fpas_lexer::Span;

impl LoweringContext {
    /// Convert through typed storage when an operand has a different IR type.
    pub(in crate::lowering) fn value_as(
        &mut self,
        value: ValueId,
        ty: TypeId,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        if self.lowered_value_type(value) == Some(ty) {
            return Ok(value);
        }
        let local = self.declare_hidden_local(ty, span)?;
        self.write_local(local, value, span)?;
        self.emit_value(Operation::ReadLocal(local), ty, span)
    }
}
