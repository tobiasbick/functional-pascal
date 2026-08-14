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
                    let replacement = if super::super::mutation::is_function_type(
                        self.executable.executable(),
                        target.expected_type,
                    ) {
                        if catalog_fallback {
                            self.prepare_catalog_routine(expression, target.expected_type, limits)?
                        } else {
                            self.finish_function_replacement(
                                task_id,
                                expression,
                                evaluated_replacement.into_iter().next(),
                                target.expected_type,
                                frame_id,
                                limits,
                            )?
                        }
                    } else {
                        evaluated_replacement
                            .into_iter()
                            .next()
                            .ok_or_else(replacement_unavailable)?
                    };
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

    fn replacement_suffix(
        executable: &fpas_bytecode::Executable,
        resolved: &super::super::mutation::ResolvedAssignment,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> Result<Vec<DebugExpression>, DebugSessionError> {
        match resolved {
            super::super::mutation::ResolvedAssignment::Existing { target, .. }
                if super::super::mutation::is_function_type(executable, target.expected_type) =>
            {
                let source = super::super::mutation::function_value_source(expression, limits)?;
                Ok(vec![DebugExpression::Name(source.requested().to_string())])
            }
            _ => Ok(vec![expression.clone()]),
        }
    }

    fn function_source_allows_catalog(
        executable: &fpas_bytecode::Executable,
        resolved: &super::super::mutation::ResolvedAssignment,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> bool {
        matches!(
            resolved,
            super::super::mutation::ResolvedAssignment::Existing { target, .. }
                if super::super::mutation::is_function_type(executable, target.expected_type)
        ) && super::super::mutation::function_value_source(expression, limits).is_ok()
    }

    fn validate_replacement_source(
        executable: &fpas_bytecode::Executable,
        resolved: &super::super::mutation::ResolvedAssignment,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> Result<(), DebugSessionError> {
        match resolved {
            super::super::mutation::ResolvedAssignment::Existing { target, .. } => {
                if super::super::mutation::is_function_type(executable, target.expected_type) {
                    super::super::mutation::function_value_source(expression, limits)?;
                }
                Ok(())
            }
            super::super::mutation::ResolvedAssignment::Transition { spec, .. } => {
                if super::super::mutation::is_function_type(executable, spec.payload_type) {
                    return Err(super::super::mutation::inactive_function_payload());
                }
                Ok(())
            }
        }
    }

    fn evaluate_replacement_for_target(
        &self,
        task_id: u64,
        expected: fpas_bytecode::DebugTypeId,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        if super::super::mutation::is_function_type(self.executable.executable(), expected) {
            self.finish_function_replacement(task_id, expression, None, expected, frame_id, limits)
        } else {
            self.evaluate_runtime_value(expression, frame_id, limits)
        }
    }

    fn finish_function_replacement(
        &self,
        task_id: u64,
        expression: &DebugExpression,
        evaluated: Option<fpas_bytecode::Value>,
        expected: fpas_bytecode::DebugTypeId,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        match super::super::mutation::function_value_source(expression, limits)? {
            super::super::mutation::FunctionSource::BindingOrRoutine(name) => {
                let value = match evaluated {
                    Some(value) => Ok(value),
                    None => self.evaluate_runtime_value(
                        &DebugExpression::Name(name.clone()),
                        frame_id,
                        limits,
                    ),
                };
                match value {
                    Ok(value) => self.prepare_function_replacement(
                        task_id, &name, value, expected, frame_id, limits,
                    ),
                    Err(error) if error.kind == DebugErrorKind::UnknownName => {
                        super::super::mutation::prepare_routine_value(
                            &self.executable,
                            &name,
                            expected,
                            limits,
                        )
                    }
                    Err(error) => Err(error),
                }
            }
            super::super::mutation::FunctionSource::Routine(_) => {
                self.prepare_catalog_routine(expression, expected, limits)
            }
        }
    }

    fn prepare_catalog_routine(
        &self,
        expression: &DebugExpression,
        expected: fpas_bytecode::DebugTypeId,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        let source = super::super::mutation::function_value_source(expression, limits)?;
        super::super::mutation::prepare_routine_value(
            &self.executable,
            source.requested(),
            expected,
            limits,
        )
    }

    fn prepare_function_replacement(
        &self,
        task_id: u64,
        name: &str,
        value: fpas_bytecode::Value,
        expected: fpas_bytecode::DebugTypeId,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        super::super::mutation::prepare_function_value(
            self.executable.executable(),
            inspection,
            name,
            value,
            expected,
            frame_id,
            limits,
        )
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
