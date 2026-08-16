//! Replacement of one unconsumed retained task result at an all-stop boundary.

use fpas_bytecode::{DebugType, ReturnConvention, Value};

use super::*;
use crate::vm::debug::completed_result::DebugTaskResultReplacement;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::tasks::CompletedResultTargetError;
use crate::vm::tasks::RetainedResultReplacement;

impl DebugSession {
    /// Replace one completed retained task result without consuming it.
    ///
    /// The expression is evaluated in `frame_id`, or in the currently selected
    /// inspection task when no frame is supplied. Ordinary popped call frames
    /// have no stable completion identity and are not accepted by this method.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a stable task, availability, convention, evaluation, type, or
    /// resource error. Failure leaves the result and inspection generation unchanged.
    pub fn replace_completed_task_result(
        &mut self,
        task_id: u64,
        frame_id: Option<u64>,
        expression: Option<&DebugExpression>,
    ) -> Result<DebugTaskResultReplacement, DebugSessionError> {
        self.replace_completed_task_result_with_limits(
            task_id,
            frame_id,
            expression,
            DebugEvaluationLimits::default(),
        )
    }

    /// Replace one completed retained task result under explicit evaluation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::replace_completed_task_result`].
    pub fn replace_completed_task_result_with_limits(
        &mut self,
        task_id: u64,
        frame_id: Option<u64>,
        expression: Option<&DebugExpression>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugTaskResultReplacement, DebugSessionError> {
        self.require_inspectable("task.result.replace")?;
        let function = match self.runtime.completed_result_function(task_id) {
            Ok(function) => function,
            Err(CompletedResultTargetError::UnknownTask) => return Err(unknown_task(task_id)),
            Err(CompletedResultTargetError::NotCompleted) => {
                return Err(result_unsupported(
                    format!("task {task_id} has not completed successfully"),
                    "Select a completed retained task whose result has not been consumed.",
                ));
            }
            Err(CompletedResultTargetError::NotRetained) => {
                return Err(result_unsupported(
                    format!("task {task_id} is detached and has no retained result"),
                    "Spawn a retained task when its result must remain replaceable.",
                ));
            }
        };
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(function.get()))
            .ok_or_else(|| {
                result_unsupported(
                    "completed task result metadata is missing",
                    "Rebuild the executable with the current compiler and retry.",
                )
            })?;
        let result_type = info.debug.result_type.ok_or_else(|| {
            result_unsupported(
                "completed task has no portable result type metadata",
                "Rebuild the executable with the current compiler and retry.",
            )
        })?;
        require_result_convention(info.return_convention, expression)?;
        reject_result_category(&self.executable, result_type)?;
        let evaluation_task = match frame_id {
            Some(frame_id) => self.task_for_frame(Some(frame_id))?,
            None => self.inspection_task_id,
        };
        let value = match expression {
            Some(expression) => self.evaluate_runtime_value(expression, frame_id, limits)?,
            None => Value::Unit,
        };
        crate::vm::debug::mutation::validate_value(
            &self.executable,
            result_type,
            &value,
            limits.max_depth,
        )
        .map_err(map_result_type_error)?;
        let prepared = self
            .inspections
            .get(&evaluation_task)
            .ok_or_else(|| unknown_task(evaluation_task))?
            .prepare_evaluation_result(&value, limits)?;
        let reserved_handles = prepared.reserved_handles();
        match self.runtime.replace_completed_result(task_id, value) {
            RetainedResultReplacement::Replaced => {}
            RetainedResultReplacement::Consumed => {
                return Err(result_unsupported(
                    format!("task {task_id} result was already consumed"),
                    "Replace a retained result before any task wait consumes it.",
                ));
            }
            RetainedResultReplacement::Pending => {
                return Err(result_unsupported(
                    format!("task {task_id} result is still pending"),
                    "Wait until the retained task completes before replacing its result.",
                ));
            }
            RetainedResultReplacement::Failed => {
                return Err(result_unsupported(
                    format!("task {task_id} retains a failure instead of a result"),
                    "Recover the failed task entry before replacing its completed result.",
                ));
            }
            RetainedResultReplacement::Unknown => return Err(unknown_task(task_id)),
        }
        self.invalidate_inspection();
        self.refresh_inspection_with_reserved_handles(Some((evaluation_task, reserved_handles)));
        self.inspection_task_id = evaluation_task;
        let Some(inspection) = self.inspections.get_mut(&evaluation_task) else {
            unreachable!("result replacement retains its evaluation task snapshot")
        };
        let rendered = inspection.retain_prepared_evaluation_result(prepared);
        Ok(DebugTaskResultReplacement {
            task_id,
            value: rendered.value,
            type_name: rendered.type_name,
            variables_reference: rendered.variables_reference,
            named_variables: rendered.named_variables,
            indexed_variables: rendered.indexed_variables,
        })
    }
}

fn require_result_convention(
    convention: ReturnConvention,
    expression: Option<&DebugExpression>,
) -> Result<(), DebugSessionError> {
    match (convention, expression.is_some()) {
        (ReturnConvention::Unit, true) => Err(result_unsupported(
            "completed procedure task does not accept a replacement expression",
            "Omit `expression`; procedure tasks retain the unit result.",
        )),
        (ReturnConvention::Value, false) => Err(result_unsupported(
            "completed function task requires a replacement expression",
            "Supply one expression matching the task function's declared result type.",
        )),
        _ => Ok(()),
    }
}

fn reject_result_category(
    executable: &fpas_bytecode::VerifiedExecutable,
    result_type: fpas_bytecode::DebugTypeId,
) -> Result<(), DebugSessionError> {
    let ty = executable
        .executable()
        .debug_types
        .get(result_type.get() as usize)
        .ok_or_else(|| {
            result_unsupported(
                "completed task result type metadata is missing",
                "Rebuild the executable with the current compiler and retry.",
            )
        })?;
    match ty {
        DebugType::Dynamic
        | DebugType::Function { .. }
        | DebugType::Task(_)
        | DebugType::Cell(_) => Err(result_unsupported(
            format!("completed task result replacement does not support {ty:?}"),
            "Use a statically declared scalar or supported aggregate result type.",
        )),
        _ => Ok(()),
    }
}

fn map_result_type_error(error: DebugSessionError) -> DebugSessionError {
    if error.kind == DebugErrorKind::VariableValueType {
        DebugSessionError {
            kind: DebugErrorKind::TaskResultReplacementType,
            message: error
                .message
                .replacen("debug replacement value", "completed task result", 1),
            hint: "Use an expression whose complete value matches the task function's declared result type."
                .to_string(),
        }
    } else {
        error
    }
}

fn result_unsupported(message: impl Into<String>, hint: impl Into<String>) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::TaskResultReplacementUnsupported,
        message: message.into(),
        hint: hint.into(),
    }
}
