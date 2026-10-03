//! Left-to-right boolean evaluation, with short-circuit branches for and/or.
//!
//! Documentation: `docs/pascal/language/basics/operators.md`.

use crate::CompileError;
use crate::lowering::context::{LoweringContext, target};
use crate::lowering::types;
use fpas_ir::{BinaryOperation, Operation, Terminator, ValueId};
use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Expr};

impl LoweringContext {
    /// Lowers a boolean chain without evaluating a skipped operand.
    pub(super) fn lower_boolean(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        if op == BinaryOp::Xor {
            return self.lower_direct_binary(
                BinaryOperation::NotEqual,
                left,
                right,
                types::BOOLEAN,
                span,
            );
        }
        let left = self.lower_expression(left)?;
        let result = self.declare_hidden_local(types::BOOLEAN, span)?;
        self.emit_effect(
            Operation::WriteLocal {
                local: result,
                value: left,
            },
            span,
        )?;
        let evaluate = self.new_block(span)?;
        let merge = self.new_block(span)?;
        let (yes, no) = if op == BinaryOp::And {
            (evaluate, merge)
        } else {
            (merge, evaluate)
        };
        self.terminate(Terminator::Branch {
            condition: left,
            then_target: target(yes),
            else_target: target(no),
        })?;
        self.switch_to(evaluate);
        let right = self.lower_expression(right)?;
        self.emit_effect(
            Operation::WriteLocal {
                local: result,
                value: right,
            },
            span,
        )?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(result), types::BOOLEAN, span)
    }
}
