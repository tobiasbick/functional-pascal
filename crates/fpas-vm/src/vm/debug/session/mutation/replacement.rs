//! Function and task replacement preparation for stopped-state mutation.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};

impl DebugSession {
    pub(super) fn replacement_suffix(
        executable: &fpas_bytecode::Executable,
        resolved: &super::super::super::mutation::ResolvedAssignment,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> Result<Vec<DebugExpression>, DebugSessionError> {
        match resolved {
            super::super::super::mutation::ResolvedAssignment::Existing { target, .. }
                if super::super::super::mutation::is_function_type(
                    executable,
                    target.expected_type,
                ) =>
            {
                let source =
                    super::super::super::mutation::function_value_source(expression, limits)?;
                Ok(vec![DebugExpression::Name(source.requested().to_string())])
            }
            super::super::super::mutation::ResolvedAssignment::Existing { target, .. }
                if super::super::super::mutation::is_task_type(
                    executable,
                    target.expected_type,
                ) =>
            {
                let source = super::super::super::mutation::task_value_source(expression, limits)?;
                Ok(vec![DebugExpression::Name(source)])
            }
            _ => Ok(vec![expression.clone()]),
        }
    }

    pub(super) fn function_source_allows_catalog(
        executable: &fpas_bytecode::Executable,
        resolved: &super::super::super::mutation::ResolvedAssignment,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> bool {
        matches!(
            resolved,
            super::super::super::mutation::ResolvedAssignment::Existing { target, .. }
                if super::super::super::mutation::is_function_type(executable, target.expected_type)
        ) && super::super::super::mutation::function_value_source(expression, limits).is_ok()
    }

    pub(super) fn validate_replacement_source(
        executable: &fpas_bytecode::Executable,
        resolved: &super::super::super::mutation::ResolvedAssignment,
        expression: &DebugExpression,
        limits: DebugEvaluationLimits,
    ) -> Result<(), DebugSessionError> {
        match resolved {
            super::super::super::mutation::ResolvedAssignment::Existing { target, .. } => {
                if super::super::super::mutation::is_function_type(executable, target.expected_type)
                {
                    super::super::super::mutation::function_value_source(expression, limits)?;
                } else if super::super::super::mutation::is_task_type(
                    executable,
                    target.expected_type,
                ) {
                    super::super::super::mutation::task_value_source(expression, limits)?;
                }
                Ok(())
            }
            super::super::super::mutation::ResolvedAssignment::Transition { spec, .. } => {
                if super::super::super::mutation::is_function_type(executable, spec.payload_type) {
                    return Err(super::super::super::mutation::inactive_function_payload());
                }
                if super::super::super::mutation::is_task_type(executable, spec.payload_type) {
                    return Err(super::super::super::mutation::inactive_task_payload());
                }
                Ok(())
            }
        }
    }

    pub(super) fn evaluate_replacement_for_target(
        &self,
        task_id: u64,
        expected: fpas_bytecode::DebugTypeId,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        if super::super::super::mutation::is_function_type(self.executable.executable(), expected) {
            self.finish_function_replacement(task_id, expression, None, expected, frame_id, limits)
        } else if super::super::super::mutation::is_task_type(
            self.executable.executable(),
            expected,
        ) {
            self.finish_task_replacement(task_id, expression, None, expected, frame_id, limits)
        } else {
            self.evaluate_runtime_value(expression, frame_id, limits)
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "existing replacement finish keeps function, task, and ordinary commit inputs together"
    )]
    pub(super) fn finish_existing_replacement(
        &self,
        task_id: u64,
        expression: &DebugExpression,
        evaluated_replacement: Vec<fpas_bytecode::Value>,
        catalog_fallback: bool,
        expected: fpas_bytecode::DebugTypeId,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        if super::super::super::mutation::is_function_type(self.executable.executable(), expected) {
            if catalog_fallback {
                self.prepare_catalog_routine(expression, expected, limits)
            } else {
                self.finish_function_replacement(
                    task_id,
                    expression,
                    evaluated_replacement.into_iter().next(),
                    expected,
                    frame_id,
                    limits,
                )
            }
        } else if super::super::super::mutation::is_task_type(
            self.executable.executable(),
            expected,
        ) {
            self.finish_task_replacement(
                task_id,
                expression,
                evaluated_replacement.into_iter().next(),
                expected,
                frame_id,
                limits,
            )
        } else {
            evaluated_replacement
                .into_iter()
                .next()
                .ok_or_else(super::replacement_unavailable)
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
        match super::super::super::mutation::function_value_source(expression, limits)? {
            super::super::super::mutation::FunctionSource::BindingOrRoutine(name) => {
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
                        super::super::super::mutation::prepare_routine_value(
                            &self.executable,
                            &name,
                            expected,
                            limits,
                        )
                    }
                    Err(error) => Err(error),
                }
            }
            super::super::super::mutation::FunctionSource::Routine(_) => {
                self.prepare_catalog_routine(expression, expected, limits)
            }
        }
    }

    fn finish_task_replacement(
        &self,
        task_id: u64,
        expression: &DebugExpression,
        evaluated: Option<fpas_bytecode::Value>,
        expected: fpas_bytecode::DebugTypeId,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        let name = super::super::super::mutation::task_value_source(expression, limits)?;
        let value = match evaluated {
            Some(value) => value,
            None => {
                self.evaluate_runtime_value(&DebugExpression::Name(name.clone()), frame_id, limits)?
            }
        };
        self.prepare_task_replacement(task_id, &name, value, expected, frame_id, limits)
    }

    fn prepare_catalog_routine(
        &self,
        expression: &DebugExpression,
        expected: fpas_bytecode::DebugTypeId,
        limits: DebugEvaluationLimits,
    ) -> Result<fpas_bytecode::Value, DebugSessionError> {
        let source = super::super::super::mutation::function_value_source(expression, limits)?;
        super::super::super::mutation::prepare_routine_value(
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
        super::super::super::mutation::prepare_function_value(
            self.executable.executable(),
            inspection,
            name,
            value,
            expected,
            frame_id,
            limits,
        )
    }

    fn prepare_task_replacement(
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
        super::super::super::mutation::prepare_task_value(
            self.executable.executable(),
            inspection,
            name,
            value,
            expected,
            frame_id,
            limits,
        )
    }
}
