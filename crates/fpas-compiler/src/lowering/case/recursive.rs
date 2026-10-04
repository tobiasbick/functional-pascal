//! Ordered case arms sharing recursive pattern matching with value decisions.

use fpas_ir::{Constant, Operation, Terminator};
use fpas_lexer::Span;
use fpas_parser::{CaseArm, Expr, Stmt};

use super::super::context::{LoweringContext, target};
use super::super::types;
use crate::CompileError;

impl LoweringContext {
    /// Lower one statement case using the same checked patterns as value decisions.
    pub(in crate::lowering) fn lower_case(
        &mut self,
        expression: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        span: Span,
    ) -> Result<(), CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let exhaustive = self
            .exhaustive_cases
            .contains(&fpas_sema::expr_lookup_key(expression));
        let value = self.lower_expression(expression)?;
        let local = self.declare_hidden_local(ty, span)?;
        self.write_local(local, value, span)?;
        let merge = self.new_block(span)?;
        let mut continues = false;
        for arm in arms {
            let next_arm = self.new_block(arm.span)?;
            let body = self.new_block(arm.span)?;
            self.begin_scope();
            if let Some(pattern) = arm.labels.first() {
                self.declare_pattern_bindings(pattern)?;
            }
            for pattern in &arm.labels {
                let next_label = self.new_block(arm.span)?;
                let value = self.emit_value(Operation::ReadLocal(local), ty, arm.span)?;
                self.lower_pattern(value, pattern, next_label)?;
                self.jump(body)?;
                self.switch_to(next_label);
            }
            self.jump(next_arm)?;
            self.switch_to(body);
            if let Some(guard) = &arm.guard {
                let condition = self.lower_expression(guard)?;
                let guarded = self.new_block(arm.span)?;
                self.terminate(Terminator::Branch {
                    condition,
                    then_target: target(guarded),
                    else_target: target(next_arm),
                })?;
                self.switch_to(guarded);
            }
            self.lower_statement(&arm.body)?;
            if !self.is_terminated() {
                self.jump(merge)?;
                continues = true;
            }
            self.end_scope();
            self.switch_to(next_arm);
        }
        if let Some(body) = else_body {
            self.begin_scope();
            self.lower_statements(body)?;
            self.end_scope();
        }
        if !self.is_terminated() {
            if else_body.is_none() && exhaustive {
                let message = self.emit_value(
                    Operation::Const(Constant::String(
                        "exhaustive case reached no pattern".into(),
                    )),
                    types::STRING,
                    span,
                )?;
                self.terminate(Terminator::Panic(message))?;
            } else {
                self.jump(merge)?;
                continues = true;
            }
        }
        if continues {
            self.switch_to(merge);
        }
        Ok(())
    }
}
