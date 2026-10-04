//! Lazy value branches with one result local and shared recursive pattern lowering.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use super::super::context::{LoweringContext, target};
use super::super::types;
use crate::CompileError;
use fpas_ir::{Constant, Operation, Terminator, TypeId, ValueId};
use fpas_parser::{CaseExpr, IfExpr};

impl LoweringContext {
    /// Select one branch and restore its value from a shared merge local.
    pub(super) fn lower_if_expression(
        &mut self,
        decision: &IfExpr,
        result: TypeId,
    ) -> Result<ValueId, CompileError> {
        let span = decision.span;
        let local = self.declare_hidden_local(result, span)?;
        let merge = self.new_block(span)?;
        for (condition, value) in std::iter::once((&decision.condition, &decision.then_value))
            .chain(
                decision
                    .elsif_values
                    .iter()
                    .map(|(condition, value)| (condition, value)),
            )
        {
            let condition = self.lower_expression(condition)?;
            let body = self.new_block(span)?;
            let next = self.new_block(span)?;
            self.terminate(Terminator::Branch {
                condition,
                then_target: target(body),
                else_target: target(next),
            })?;
            self.switch_to(body);
            let value = self.lower_expression_as(value, result)?;
            let value = self.coerce_value_type(value, result, span)?;
            self.write_local(local, value, span)?;
            self.jump(merge)?;
            self.switch_to(next);
        }
        let value = self.lower_expression_as(&decision.else_value, result)?;
        let value = self.coerce_value_type(value, result, span)?;
        self.write_local(local, value, span)?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(local), result, span)
    }

    /// Evaluate the scrutinee once and lower scoped patterns and lazy branch values.
    pub(super) fn lower_case_expression(
        &mut self,
        decision: &CaseExpr,
        result: TypeId,
    ) -> Result<ValueId, CompileError> {
        let span = decision.span;
        let matched_type = self.expression_ir_type(&decision.value)?;
        let matched = self.lower_expression(&decision.value)?;
        let scrutinee = self.declare_hidden_local(matched_type, span)?;
        self.write_local(scrutinee, matched, span)?;
        let result_local = self.declare_hidden_local(result, span)?;
        let merge = self.new_block(span)?;
        for arm in &decision.arms {
            let next_arm = self.new_block(arm.span)?;
            let body = self.new_block(arm.span)?;
            self.begin_scope();
            if let Some(pattern) = arm.labels.first() {
                self.declare_pattern_bindings(pattern)?;
            }
            for pattern in &arm.labels {
                let next_label = self.new_block(arm.span)?;
                let matched =
                    self.emit_value(Operation::ReadLocal(scrutinee), matched_type, arm.span)?;
                self.lower_pattern(matched, pattern, next_label)?;
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
            let value = self.lower_expression_as(&arm.body, result)?;
            let value = self.coerce_value_type(value, result, arm.span)?;
            self.write_local(result_local, value, arm.span)?;
            self.jump(merge)?;
            self.end_scope();
            self.switch_to(next_arm);
        }
        if let Some(value) = &decision.else_value {
            let value = self.lower_expression_as(value, result)?;
            let value = self.coerce_value_type(value, result, span)?;
            self.write_local(result_local, value, span)?;
            self.jump(merge)?;
        } else {
            let message = self.emit_value(
                Operation::Const(Constant::String(
                    "exhaustive case expression reached no pattern".into(),
                )),
                types::STRING,
                span,
            )?;
            self.terminate(Terminator::Panic(message))?;
        }
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(result_local), result, span)
    }
}
