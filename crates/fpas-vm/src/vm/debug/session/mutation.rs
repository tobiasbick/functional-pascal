//! Stopped-state orchestration for one protocol-neutral variable update.

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluateResult, DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{DebugAssignmentSelector, DebugAssignmentTarget};

impl DebugSession {
    /// Replace one mutable variable or supported descendant with an evaluated expression.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, expression, type, resource, or availability error. A
    /// failure leaves live program state and the current inspection generation unchanged.
    pub fn set_variable(
        &mut self,
        variables_reference: u64,
        name: &str,
        expression: &DebugExpression,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.set_variable_with_limits(
            variables_reference,
            name,
            expression,
            DebugEvaluationLimits::default(),
        )
    }

    /// Replace one variable using explicit expression and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::set_variable`].
    pub fn set_variable_with_limits(
        &mut self,
        variables_reference: u64,
        name: &str,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.require_stopped("variable.set")?;
        let result = (|| {
            let generation = (variables_reference >> 32) as u32;
            let (task_id, target) = self
                .inspections
                .iter()
                .find_map(|(&task_id, inspection)| {
                    (inspection.generation() == generation).then(|| {
                        inspection
                            .resolve_mutation_target(variables_reference, name)
                            .map(|target| (task_id, target))
                    })
                })
                .ok_or_else(|| DebugSessionError {
                    kind: DebugErrorKind::VariableTargetExpired,
                    message: format!(
                        "debug variable target `{name}` belongs to an expired stop snapshot"
                    ),
                    hint: "Request scopes and variables again for the current stop.".to_string(),
                })??;
            let replacement = self.evaluate_runtime_value(expression, target.frame_id, limits)?;
            self.commit_mutation(task_id, &target, replacement, limits)
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Replace one textual mutable target with an evaluated expression.
    ///
    /// # Errors
    ///
    /// Returns a stable state, frame, target, selector, expression, type, resource, or
    /// availability error. Failure leaves live state and inspection handles unchanged.
    pub fn set_expression(
        &mut self,
        target: &DebugAssignmentTarget,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.set_expression_with_limits(
            target,
            expression,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Replace one textual target with explicit expression and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::set_expression`].
    pub fn set_expression_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.require_stopped("expression.set")?;
        let result = (|| {
            let task_id = self.task_for_frame(frame_id)?;
            let (target, current) = self
                .inspections
                .get(&task_id)
                .ok_or_else(|| unknown_task(task_id))?
                .resolve_named_mutation_target(frame_id, &assignment.root)?;
            let (indexes, replacement) =
                self.evaluate_assignment_inputs(assignment, expression, frame_id, limits)?;
            let target = super::super::mutation::resolve_target(
                self.executable.executable(),
                assignment,
                target,
                current,
                &indexes,
            )?;
            self.commit_mutation(task_id, &target, replacement, limits)
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn evaluate_assignment_inputs(
        &self,
        assignment: &DebugAssignmentTarget,
        replacement: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<(Vec<fpas_bytecode::Value>, fpas_bytecode::Value), DebugSessionError> {
        let mut expressions = assignment
            .selectors
            .iter()
            .filter_map(|selector| match selector {
                DebugAssignmentSelector::Field(_) => None,
                DebugAssignmentSelector::Index(expression) => Some(expression.clone()),
            })
            .collect::<Vec<_>>();
        expressions.push(replacement.clone());
        let mut values = self.evaluate_runtime_values(&expressions, frame_id, limits)?;
        let replacement = values.pop().ok_or_else(|| DebugSessionError {
            kind: DebugErrorKind::VariableUnavailable,
            message: "debug assignment replacement value is unavailable".to_string(),
            hint: "Retry the mutation with one complete replacement expression.".to_string(),
        })?;
        Ok((values, replacement))
    }

    fn commit_mutation(
        &mut self,
        task_id: u64,
        target: &super::super::inspection::MutationTarget,
        replacement: fpas_bytecode::Value,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        super::super::mutation::validate_replacement(
            &self.executable,
            target,
            &replacement,
            limits.max_depth,
        )?;
        let generation = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .generation();
        let worker = self
            .runtime
            .worker_mut(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let committed = super::super::mutation::commit(worker, generation, target, replacement)?;
        self.invalidate_inspection();
        self.refresh_inspection();
        self.inspection_task_id = task_id;
        self.inspections
            .get_mut(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .retain_evaluation_result(committed, limits)
    }
}
