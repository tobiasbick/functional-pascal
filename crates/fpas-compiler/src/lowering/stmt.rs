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
                let key = fpas_sema::designator_lookup_key(target);
                if let Some(info) = self.event_writes.get(&key).cloned() {
                    return self.lower_event_write(target, value, &info, *span);
                }
                if let Some(info) = self.property_writes.get(&key).cloned() {
                    return self.lower_property_write(target, value, &info, *span);
                }
                let value = self.lower_expression(value)?;
                self.lower_designator_write(target, value, *span)
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
                let call_key = fpas_sema::designator_lookup_key(designator);
                if let Some(info) = self.event_raises.get(&call_key).cloned() {
                    let _ = self.lower_event_raise(designator, args, &info, *span)?;
                    return Ok(());
                }
                let result = if self.intrinsic_calls.contains_key(&call_key) {
                    Some(super::types::UNIT)
                } else {
                    self.member_call_result(call_key).or_else(|| {
                        let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
                            return None;
                        };
                        self.call_result_type(name)
                    })
                }
                .ok_or_else(|| unsupported(designator.span, "unresolved procedure call"))?;
                let _ = self.lower_call(designator, args, result, *span, call_key)?;
                Ok(())
            }
            Stmt::Expression { span, .. } => Err(unsupported(*span, "effect expression")),
            Stmt::Go { expr, span } => self.lower_go(expr, *span, false).map(|_| ()),
        }
    }

    pub(super) fn lower_statements(&mut self, statements: &[Stmt]) -> Result<(), CompileError> {
        for statement in statements {
            self.lower_statement(statement)?;
        }
        Ok(())
    }

    fn lower_variable(&mut self, definition: &VarDef, mutable: bool) -> Result<(), CompileError> {
        let ty = self.declared_type(&definition.type_expr)?;
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
