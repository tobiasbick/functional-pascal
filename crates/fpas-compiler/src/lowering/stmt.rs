//! Scalar declarations, assignments, blocks, returns, and panic lowering.

use fpas_ir::{Operation, Terminator};
use fpas_parser::{DesignatorPart, Stmt, VarDef};

use crate::CompileError;

use super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(super) fn lower_statement(&mut self, statement: &Stmt) -> Result<(), CompileError> {
        if self.is_terminated() {
            return Ok(());
        }
        match statement {
            Stmt::Block(statements, _) => {
                self.begin_scope();
                for statement in statements {
                    self.lower_statement(statement)?;
                }
                self.end_scope();
                Ok(())
            }
            Stmt::Var(definition) => self.lower_variable(definition, false),
            Stmt::MutableVar(definition) => self.lower_variable(definition, true),
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                let [DesignatorPart::Ident(name, _)] = target.parts.as_slice() else {
                    return Err(unsupported(target.span, "aggregate assignment"));
                };
                let value = self.lower_expression(value)?;
                self.write_named_local(name, value, *span)
            }
            Stmt::Return(value, _span) => {
                let value = value
                    .as_ref()
                    .map(|value| self.lower_expression(value))
                    .transpose()?;
                self.terminate(Terminator::Return(value))
            }
            Stmt::Panic(value, span) => {
                let value = self.lower_expression(value)?;
                self.set_last_instruction_source(*span)?;
                self.terminate(Terminator::Panic(value))
            }
            Stmt::If { .. }
            | Stmt::While { .. }
            | Stmt::Repeat { .. }
            | Stmt::For { .. }
            | Stmt::Case { .. }
            | Stmt::Break(_)
            | Stmt::Continue(_) => self.lower_control_flow(statement),
            Stmt::ForIn { span, .. } => Err(unsupported(*span, "for-in loop")),
            Stmt::Call {
                designator,
                args,
                span,
            } => {
                let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
                    return Err(unsupported(designator.span, "method or qualified call"));
                };
                let result = self
                    .call_result_type(name)
                    .ok_or_else(|| unsupported(designator.span, "unresolved procedure call"))?;
                let _ = self.lower_call(designator, args, result, *span)?;
                Ok(())
            }
            Stmt::Expression { span, .. } => Err(unsupported(*span, "effect expression")),
            Stmt::Go { span, .. } => Err(unsupported(*span, "task statement")),
        }
    }

    pub(super) fn lower_statements(&mut self, statements: &[Stmt]) -> Result<(), CompileError> {
        for statement in statements {
            self.lower_statement(statement)?;
        }
        Ok(())
    }

    fn lower_variable(&mut self, definition: &VarDef, mutable: bool) -> Result<(), CompileError> {
        let ty = self.expression_ir_type(&definition.value)?;
        let value = self.lower_expression(&definition.value)?;
        if mutable && self.is_cell_backed(&definition.name) {
            let cell_ty = self.cell_type(ty, definition.span)?;
            let cell = self.emit_value(Operation::MakeCell(value), cell_ty, definition.span)?;
            let local = self.declare_local(&definition.name, cell_ty, true, definition.span)?;
            self.mark_binding_cell(&definition.name, ty);
            self.emit_effect(
                Operation::WriteLocal { value: cell, local },
                definition.span,
            )
        } else {
            let local = self.declare_local(&definition.name, ty, mutable, definition.span)?;
            self.emit_effect(Operation::WriteLocal { value, local }, definition.span)
        }
    }
}
