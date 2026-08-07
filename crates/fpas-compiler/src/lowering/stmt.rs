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
                let (local, _) = self.resolve_local(name, target.span)?;
                let value = self.lower_expression(value)?;
                self.write_local(local, value, *span)
            }
            Stmt::Return(value, span) => {
                if value.is_some() {
                    return Err(unsupported(*span, "value return from root entry"));
                }
                self.terminate(Terminator::Return(None))
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
            Stmt::Call { span, .. } => Err(unsupported(*span, "procedure call")),
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
        let local = self.declare_local(&definition.name, ty, mutable, definition.span)?;
        self.emit_effect(Operation::WriteLocal { value, local }, definition.span)
    }
}
