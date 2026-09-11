//! Preserve evaluated operands across expression-generated control flow.
//!
//! `docs/pascal/language/error-handling/try.md` describes early propagation through `try`.

use fpas_ir::{BlockId, Instruction, Operation, TypeId, ValueId};
use fpas_lexer::Span;
use fpas_parser::Expr;

use crate::CompileError;

use super::{LoweringContext, unsupported};

/// An evaluated operand and the block in which it is available.
pub(in crate::lowering) struct SavedValue {
    value: ValueId,
    block: BlockId,
}

impl LoweringContext {
    /// Remembers an operand without adding instructions on the straight-line path.
    pub(in crate::lowering) fn save_value(&self, value: ValueId) -> SavedValue {
        SavedValue {
            value,
            block: self.current,
        }
    }

    /// Makes a saved operand available after lowering subsequent expressions.
    pub(in crate::lowering) fn restore_value(
        &mut self,
        saved: SavedValue,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        if saved.block == self.current {
            return Ok(saved.value);
        }
        let ty = self
            .lowered_value_type(saved.value)
            .ok_or_else(|| unsupported(span, "saved operand type"))?;
        let local = self.declare_hidden_local(ty, span)?;
        let block = self
            .blocks
            .get_mut(saved.block.get() as usize)
            .filter(|block| block.id == saved.block)
            .ok_or_else(|| unsupported(span, "saved operand block"))?;
        // Append before the separate terminator, keeping existing instruction/debug indices intact.
        // SSA values are immutable: this preserves the evaluated value, not a later local read.
        block.instructions.push(Instruction {
            source: None,
            result: None,
            operation: Operation::WriteLocal {
                local,
                value: saved.value,
            },
        });
        self.emit_value(Operation::ReadLocal(local), ty, span)
    }

    /// Evaluates operands in source order and restores them in the final continuation block.
    pub(in crate::lowering) fn lower_expression_values(
        &mut self,
        expressions: &[Expr],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Vec<ValueId>, CompileError> {
        let saved = expressions
            .iter()
            .map(|expression| {
                let value = match expected {
                    Some(ty) => self.lower_expression_as(expression, ty)?,
                    None => self.lower_expression(expression)?,
                };
                Ok(self.save_value(value))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        saved
            .into_iter()
            .map(|value| self.restore_value(value, span))
            .collect()
    }
}
