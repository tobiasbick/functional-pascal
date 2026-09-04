//! Inclusive counting-loop lowering without terminal counter overflow.
//!
//! Documentation: `docs/pascal/language/control-flow/for-loops.md`.

use fpas_ir::{BinaryOperation, Constant, Operation, Terminator, UnaryOperation};
use fpas_parser::{Expr, ForDirection, Stmt};

use crate::CompileError;

use super::super::context::{LoopTargets, LoweringContext, target};
use super::super::types;

impl LoweringContext {
    /// Lowers inclusive bounds with a terminal check before advancing the counter.
    pub(super) fn lower_for(
        &mut self,
        variable: &str,
        start: &Expr,
        direction: &ForDirection,
        end: &Expr,
        body: &Stmt,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let counter_type = self.expression_ir_type(start)?;
        let start_value = self.lower_expression(start)?;
        let end_value = self.lower_expression(end)?;
        self.begin_scope();
        let variable_local = self.declare_local(variable, counter_type, true, span)?;
        self.write_local(variable_local, start_value, span)?;
        let end_local = self.declare_hidden_local(counter_type, span)?;
        self.write_local(end_local, end_value, span)?;

        let condition_block = self.new_block(span)?;
        let body_block = self.new_block(span)?;
        let terminal_block = self.new_block(span)?;
        let increment_block = self.new_block(span)?;
        let after_block = self.new_block(span)?;
        self.jump(condition_block)?;

        self.switch_to(condition_block);
        let current = self.emit_value(Operation::ReadLocal(variable_local), counter_type, span)?;
        let bound = self.emit_value(Operation::ReadLocal(end_local), counter_type, span)?;
        let comparison = match (direction, counter_type == types::BOOLEAN) {
            (ForDirection::To, true) => BinaryOperation::LessEqualDynamic,
            (ForDirection::Downto, true) => BinaryOperation::GreaterEqualDynamic,
            (ForDirection::To, false) => BinaryOperation::LessEqualInteger,
            (ForDirection::Downto, false) => BinaryOperation::GreaterEqualInteger,
        };
        let condition = self.emit_binary(comparison, current, bound, types::BOOLEAN, span)?;
        self.terminate(Terminator::Branch {
            condition,
            then_target: target(body_block),
            else_target: target(after_block),
        })?;

        self.push_loop(LoopTargets {
            break_block: after_block,
            continue_block: terminal_block,
        });
        self.switch_to(body_block);
        self.lower_statement(body)?;
        if !self.is_terminated() {
            self.jump(terminal_block)?;
        }

        self.switch_to(terminal_block);
        let current = self.emit_value(Operation::ReadLocal(variable_local), counter_type, span)?;
        let bound = self.emit_value(Operation::ReadLocal(end_local), counter_type, span)?;
        let finished =
            self.emit_binary(BinaryOperation::Equal, current, bound, types::BOOLEAN, span)?;
        self.terminate(Terminator::Branch {
            condition: finished,
            then_target: target(after_block),
            else_target: target(increment_block),
        })?;

        self.switch_to(increment_block);
        let current = self.emit_value(Operation::ReadLocal(variable_local), counter_type, span)?;
        let updated = if counter_type == types::BOOLEAN {
            self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NotBoolean,
                    operand: current,
                },
                counter_type,
                span,
            )?
        } else {
            let one =
                self.emit_value(Operation::Const(Constant::Integer(1)), types::INTEGER, span)?;
            let operation = match direction {
                ForDirection::To => BinaryOperation::AddInteger,
                ForDirection::Downto => BinaryOperation::SubtractInteger,
            };
            self.emit_binary(operation, current, one, counter_type, span)?
        };
        self.write_local(variable_local, updated, span)?;
        self.jump(body_block)?;
        self.pop_loop();
        self.end_scope();
        self.switch_to(after_block);
        Ok(())
    }
}
