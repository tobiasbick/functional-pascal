//! Structured scalar control flow lowered to explicit CFG blocks.
//!
//! Documentation: `docs/pascal/language/control-flow/while-repeat.md`.

mod counting;

use fpas_ir::{BinaryOperation, Constant, IntrinsicId, IrType, Operation, Terminator};
use fpas_parser::{Expr, Stmt};

use crate::CompileError;

use super::context::{LoopTargets, LoweringContext, target, unsupported};
use super::types;

impl LoweringContext {
    /// Lowers structured branches, loops, and loop-control statements.
    pub(super) fn lower_control_flow(&mut self, statement: &Stmt) -> Result<(), CompileError> {
        match statement {
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.lower_if(condition, then_branch, else_branch.as_deref(), *span),
            Stmt::While {
                condition,
                body,
                span,
            } => self.lower_while(condition, body, *span),
            Stmt::Repeat {
                body,
                condition,
                span,
            } => self.lower_repeat(body, condition, *span),
            Stmt::For {
                var_name,
                start,
                direction,
                end,
                body,
                span,
                ..
            } => self.lower_for(var_name, start, direction, end, body, *span),
            Stmt::ForIn {
                var_name,
                iterable,
                body,
                span,
                ..
            } => self.lower_for_in(var_name, iterable, body, *span),
            Stmt::Case {
                expr,
                arms,
                else_body,
                span,
            } => self.lower_case(expr, arms, else_body.as_deref(), *span),
            Stmt::Break(span) => {
                let targets = self.loop_targets(*span)?;
                self.jump(targets.break_block)
            }
            Stmt::Continue(span) => {
                let targets = self.loop_targets(*span)?;
                self.jump(targets.continue_block)
            }
            _ => Err(unsupported(
                statement_span(statement),
                "control-flow statement",
            )),
        }
    }

    fn lower_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let condition = self.lower_expression(condition)?;
        let then_block = self.new_block(span)?;
        let else_block = self.new_block(span)?;
        let merge_block = self.new_block(span)?;
        self.terminate(Terminator::Branch {
            condition,
            then_target: target(then_block),
            else_target: target(else_block),
        })?;

        self.switch_to(then_block);
        self.lower_statement(then_branch)?;
        let then_continues = !self.is_terminated();
        if then_continues {
            self.jump(merge_block)?;
        }

        self.switch_to(else_block);
        if let Some(else_branch) = else_branch {
            self.lower_statement(else_branch)?;
        }
        let else_continues = !self.is_terminated();
        if else_continues {
            self.jump(merge_block)?;
        }

        if then_continues || else_continues {
            self.switch_to(merge_block);
        } else {
            self.remove_last_block_if(merge_block);
            self.switch_to(else_block);
        }
        Ok(())
    }

    fn lower_while(
        &mut self,
        condition: &Expr,
        body: &Stmt,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let condition_block = self.new_block(span)?;
        let body_block = self.new_block(span)?;
        let after_block = self.new_block(span)?;
        self.jump(condition_block)?;

        self.switch_to(condition_block);
        let condition = self.lower_expression(condition)?;
        self.terminate(Terminator::Branch {
            condition,
            then_target: target(body_block),
            else_target: target(after_block),
        })?;

        self.push_loop(LoopTargets {
            break_block: after_block,
            continue_block: condition_block,
        });
        self.switch_to(body_block);
        self.lower_statement(body)?;
        if !self.is_terminated() {
            self.jump(condition_block)?;
        }
        self.pop_loop();
        self.switch_to(after_block);
        Ok(())
    }

    fn lower_repeat(
        &mut self,
        body: &[Stmt],
        condition: &Expr,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let body_block = self.new_block(span)?;
        let condition_block = self.new_block(span)?;
        let after_block = self.new_block(span)?;
        self.jump(body_block)?;

        self.begin_scope();
        self.push_loop(LoopTargets {
            break_block: after_block,
            continue_block: condition_block,
        });
        self.switch_to(body_block);
        self.lower_statements(body)?;
        if !self.is_terminated() {
            self.jump(condition_block)?;
        }
        self.pop_loop();
        self.end_scope();
        self.switch_to(condition_block);
        let condition = self.lower_expression(condition)?;
        self.terminate(Terminator::Branch {
            condition,
            then_target: target(after_block),
            else_target: target(body_block),
        })?;
        self.switch_to(after_block);
        Ok(())
    }

    fn lower_for_in(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &Stmt,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        self.begin_scope();
        let iterable_ty = self.expression_ir_type(iterable)?;
        let (element_ty, collection_ty, keys) = match self.type_kind(iterable_ty) {
            Some(IrType::Array(element)) => (element, iterable_ty, false),
            Some(IrType::Dictionary { key, .. }) => (key, self.array_type(key, span)?, true),
            _ => return Err(unsupported(span, "for-in collection type")),
        };
        let mut collection = self.lower_expression(iterable)?;
        if keys {
            self.record_call_arguments(1, span)?;
            collection = self.emit_value(
                Operation::Intrinsic {
                    intrinsic: intrinsic_id(fpas_bytecode::Intrinsic::Dict(
                        fpas_bytecode::DictIntrinsic::Keys,
                    )),
                    arguments: vec![collection],
                },
                collection_ty,
                span,
            )?;
        }
        let collection_local = self.declare_hidden_local(collection_ty, span)?;
        self.write_local(collection_local, collection, span)?;
        let zero = self.emit_value(Operation::Const(Constant::Integer(0)), types::INTEGER, span)?;
        let index_local = self.declare_hidden_local(types::INTEGER, span)?;
        self.write_local(index_local, zero, span)?;
        let collection =
            self.emit_value(Operation::ReadLocal(collection_local), collection_ty, span)?;
        self.record_call_arguments(1, span)?;
        let length = self.emit_value(
            Operation::Intrinsic {
                intrinsic: intrinsic_id(fpas_bytecode::Intrinsic::Array(
                    fpas_bytecode::ArrayIntrinsic::Length,
                )),
                arguments: vec![collection],
            },
            types::INTEGER,
            span,
        )?;
        let length_local = self.declare_hidden_local(types::INTEGER, span)?;
        self.write_local(length_local, length, span)?;
        let variable_local = self.declare_local(variable, element_ty, true, span)?;

        let condition_block = self.new_block(span)?;
        let body_block = self.new_block(span)?;
        let increment_block = self.new_block(span)?;
        let after_block = self.new_block(span)?;
        self.jump(condition_block)?;
        self.switch_to(condition_block);
        let index = self.emit_value(Operation::ReadLocal(index_local), types::INTEGER, span)?;
        let length = self.emit_value(Operation::ReadLocal(length_local), types::INTEGER, span)?;
        let condition = self.emit_binary(
            BinaryOperation::LessThanInteger,
            index,
            length,
            types::BOOLEAN,
            span,
        )?;
        self.terminate(Terminator::Branch {
            condition,
            then_target: target(body_block),
            else_target: target(after_block),
        })?;

        self.push_loop(LoopTargets {
            break_block: after_block,
            continue_block: increment_block,
        });
        self.switch_to(body_block);
        let collection =
            self.emit_value(Operation::ReadLocal(collection_local), collection_ty, span)?;
        let index = self.emit_value(Operation::ReadLocal(index_local), types::INTEGER, span)?;
        let value = self.emit_value(Operation::IndexGet { collection, index }, element_ty, span)?;
        self.write_local(variable_local, value, span)?;
        self.lower_statement(body)?;
        if !self.is_terminated() {
            self.jump(increment_block)?;
        }

        self.switch_to(increment_block);
        let index = self.emit_value(Operation::ReadLocal(index_local), types::INTEGER, span)?;
        let one = self.emit_value(Operation::Const(Constant::Integer(1)), types::INTEGER, span)?;
        let next = self.emit_binary(
            BinaryOperation::AddInteger,
            index,
            one,
            types::INTEGER,
            span,
        )?;
        self.write_local(index_local, next, span)?;
        self.jump(condition_block)?;
        self.pop_loop();
        self.end_scope();
        self.switch_to(after_block);
        Ok(())
    }
}

fn intrinsic_id(intrinsic: fpas_bytecode::Intrinsic) -> IntrinsicId {
    IntrinsicId::new(u32::from(u16::from(intrinsic)))
}

fn statement_span(statement: &Stmt) -> fpas_lexer::Span {
    match statement {
        Stmt::Block(_, span)
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::Break(span)
        | Stmt::Continue(span) => *span,
        Stmt::Var(value) | Stmt::MutableVar(value) => value.span,
        Stmt::Assign { span, .. }
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => *span,
    }
}
