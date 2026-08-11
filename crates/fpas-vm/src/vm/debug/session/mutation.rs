//! Stopped-state orchestration for one protocol-neutral variable update.

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluateResult, DebugEvaluationLimits, DebugExpression};

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
            super::super::mutation::validate_replacement(
                &self.executable,
                &target,
                &replacement,
                limits.max_depth,
            )?;
            let worker = self
                .runtime
                .worker_mut(task_id)
                .ok_or_else(|| unknown_task(task_id))?;
            let committed =
                super::super::mutation::commit(worker, target.generation, &target, replacement)?;
            self.inspection_task_id = task_id;
            self.invalidate_inspection();
            self.refresh_inspection();
            self.inspections
                .get_mut(&task_id)
                .ok_or_else(|| unknown_task(task_id))?
                .retain_evaluation_result(committed, limits)
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }
}
