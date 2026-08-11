//! Stopped-state orchestration for one protocol-neutral variable update.

use super::*;

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
            let target = self
                .inspection
                .resolve_mutation_target(variables_reference, name)?;
            let replacement = self.evaluate_runtime_value(expression, target.frame_id, limits)?;
            super::super::mutation::validate_replacement(
                &self.executable,
                &target,
                &replacement,
                limits.max_depth,
            )?;
            let committed = super::super::mutation::commit(
                &mut self.worker,
                self.inspection_generation,
                &target,
                replacement,
            )?;
            self.invalidate_inspection();
            self.refresh_inspection();
            self.inspection.retain_evaluation_result(committed, limits)
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }
}
