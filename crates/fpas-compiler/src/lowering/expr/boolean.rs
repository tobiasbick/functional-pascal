//! Boolean short-circuit control flow and eager exclusive disjunction.
//!
//! Documentation: `docs/pascal/language/basics/operators.md#evaluation-order`.

use fpas_ir::{BinaryOperation, Operation, Terminator, TypeId, ValueId};
use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Expr};

use crate::CompileError;
use crate::lowering::context::{LoweringContext, target, unsupported};
use crate::lowering::types;

impl LoweringContext {
    /// Branches for Boolean `and`/`or` and evaluates both operands of `xor`.
    pub(super) fn lower_boolean(
        &mut self,
        operation: BinaryOp,
        left: &Expr,
        right: &Expr,
        result_ty: TypeId,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        match operation {
            BinaryOp::And | BinaryOp::Or => self.lower_short_circuit(operation, left, right, span),
            BinaryOp::Xor => {
                self.lower_direct_binary(BinaryOperation::NotEqual, left, right, result_ty, span)
            }
            _ => Err(unsupported(span, "boolean operation")),
        }
    }

    fn lower_short_circuit(
        &mut self,
        operation: BinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let left = self.lower_expression(left)?;
        let result = self.declare_hidden_local(types::BOOLEAN, span)?;
        self.write_local(result, left, span)?;
        let right_block = self.new_block(span)?;
        let merge = self.new_block(span)?;
        let (then_block, else_block) = if operation == BinaryOp::And {
            (right_block, merge)
        } else {
            (merge, right_block)
        };
        self.terminate(Terminator::Branch {
            condition: left,
            then_target: target(then_block),
            else_target: target(else_block),
        })?;

        self.switch_to(right_block);
        let right = self.lower_expression(right)?;
        self.write_local(result, right, span)?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(result), types::BOOLEAN, span)
    }
}
