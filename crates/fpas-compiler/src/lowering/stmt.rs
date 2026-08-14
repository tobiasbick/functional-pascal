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
                let value = match self.designator_type(target) {
                    Some(expected) => self.lower_expression_as(value, expected)?,
                    None => self.lower_expression(value)?,
                };
                self.lower_designator_write(target, value, *span)
            }
            Stmt::Return(value, _span) => {
                let value = value
                    .as_ref()
                    .map(|value| match value {
                        fpas_parser::Expr::ResultOk(inner, span) => {
                            self.lower_wrapper_as(Some(inner), 0, self.current_result_type(), *span)
                        }
                        fpas_parser::Expr::ResultError(inner, span) => {
                            self.lower_wrapper_as(Some(inner), 1, self.current_result_type(), *span)
                        }
                        fpas_parser::Expr::OptionSome(inner, span) => {
                            self.lower_wrapper_as(Some(inner), 2, self.current_result_type(), *span)
                        }
                        fpas_parser::Expr::OptionNone(span) => {
                            self.lower_wrapper_as(None, 3, self.current_result_type(), *span)
                        }
                        _ => self.lower_expression(value),
                    })
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
            | Stmt::ForIn { .. }
            | Stmt::Case { .. }
            | Stmt::Break(_)
            | Stmt::Continue(_) => self.lower_control_flow(statement),
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
                        let qualified = designator
                            .parts
                            .iter()
                            .map(|part| match part {
                                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                                DesignatorPart::Index(_, _) => None,
                            })
                            .collect::<Option<Vec<_>>>()?
                            .join(".");
                        self.call_result_type(&qualified).or_else(|| {
                            let canonical = format!("Std.Graph.{qualified}");
                            crate::intrinsic_catalog::resolve(&canonical, None)
                                .map(|_| super::types::UNIT)
                        })
                    })
                }
                .ok_or_else(|| unsupported(designator.span, "unresolved procedure call"))?;
                let _ = self.lower_call(designator, args, result, *span, call_key)?;
                Ok(())
            }
            Stmt::Expression { expr, .. } => self.lower_expression(expr).map(|_| ()),
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
        let declared = self.declared_type(&definition.type_expr)?;
        let ty = if self.is_bare_task_binding(declared) {
            let inferred = self.expression_ir_type(&definition.value)?;
            self.specialize_task_binding(declared, inferred)
        } else {
            declared
        };
        let value = self.lower_expression_as(&definition.value, ty)?;
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
