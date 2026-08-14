//! Stopped-state orchestration for one protocol-neutral variable update.

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluateResult, DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{DebugAssignmentSelector, DebugAssignmentTarget};

mod replacement;

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
            let replacement = self.evaluate_replacement_for_target(
                task_id,
                target.expected_type,
                expression,
                target.frame_id,
                limits,
            )?;
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
            if current.is_none() && !assignment.selectors.is_empty() {
                return Err(uninitialized_root_path(&assignment.root));
            }
            let current = current.unwrap_or(fpas_bytecode::Value::Unit);
            let index_expressions = assignment
                .selectors
                .iter()
                .filter_map(|selector| match selector {
                    DebugAssignmentSelector::Field(_) => None,
                    DebugAssignmentSelector::Index(expression) => Some(expression.clone()),
                })
                .collect::<Vec<_>>();
            let mut resolved_slot = None;
            let executable = Arc::clone(&self.executable);
            let eval = self.evaluate_runtime_values_with_dynamic_suffix(
                &index_expressions,
                frame_id,
                limits,
                |indexes| {
                    let resolved = super::super::mutation::resolve_assignment(
                        executable.executable(),
                        assignment,
                        target,
                        current,
                        indexes,
                    )?;
                    Self::validate_replacement_source(
                        executable.executable(),
                        &resolved,
                        expression,
                        limits,
                    )?;
                    let suffix = Self::replacement_suffix(
                        executable.executable(),
                        &resolved,
                        expression,
                        limits,
                    )?;
                    resolved_slot = Some(resolved.clone());
                    Ok((resolved, suffix))
                },
            );
            let (resolved, evaluated_replacement, catalog_fallback) = match eval {
                Ok((resolved, values)) => (resolved, values, false),
                Err(error) => match resolved_slot.take() {
                    Some(resolved)
                        if error.kind == DebugErrorKind::UnknownName
                            && Self::function_source_allows_catalog(
                                executable.executable(),
                                &resolved,
                                expression,
                                limits,
                            ) =>
                    {
                        (resolved, Vec::new(), true)
                    }
                    _ => return Err(error),
                },
            };
            let (target, replacement) = match resolved {
                super::super::mutation::ResolvedAssignment::Existing { target, .. } => {
                    let replacement = self.finish_existing_replacement(
                        task_id,
                        expression,
                        evaluated_replacement,
                        catalog_fallback,
                        target.expected_type,
                        frame_id,
                        limits,
                    )?;
                    (target, replacement)
                }
                super::super::mutation::ResolvedAssignment::Transition { target, spec, .. } => {
                    let replacement = super::super::mutation::construct_transition(
                        &self.executable,
                        spec,
                        evaluated_replacement
                            .into_iter()
                            .next()
                            .ok_or_else(replacement_unavailable)?,
                        limits,
                    )?;
                    (target, replacement)
                }
            };
            self.commit_mutation(task_id, &target, replacement, limits)
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    pub(super) fn commit_mutation(
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

fn uninitialized_root_path(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug variable target `{name}` has no writable descendants before initialization"
        ),
        hint: "Initialize the complete binding before editing fields, indexes, or payload descendants."
            .to_string(),
    }
}

fn replacement_unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug replacement expression produced no value".to_string(),
        hint: "Retry the assignment at the current stop with one supported replacement expression."
            .to_string(),
    }
}
