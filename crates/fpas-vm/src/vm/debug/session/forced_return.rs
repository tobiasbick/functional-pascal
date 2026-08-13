//! Stopped-state orchestration for one protocol-neutral forced return.

use std::sync::atomic::Ordering;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::forced_return::{
    DebugForcedReturnResult, commit, prepare_return_value, reject_declared_category,
    require_convention, require_eligible, require_result_type, unknown_frame, unsupported,
};

impl DebugSession {
    /// Complete the active ordinary callee with a validated return value and remain stopped in its caller.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a stable state, frame, task, convention, type, evaluation, or resource error. Failure
    /// leaves live program state, the current stop, and inspection handles unchanged.
    pub fn force_return(
        &mut self,
        frame_id: u64,
        expression: Option<&DebugExpression>,
    ) -> Result<DebugForcedReturnResult, DebugSessionError> {
        self.force_return_with_limits(frame_id, expression, DebugEvaluationLimits::default())
    }

    /// Complete the active callee using explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::force_return`].
    pub fn force_return_with_limits(
        &mut self,
        frame_id: u64,
        expression: Option<&DebugExpression>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugForcedReturnResult, DebugSessionError> {
        let result = (|| {
            if self.state == DebugSessionState::Failed
                || self.last_stop.reason == DebugStopReason::RuntimeError
            {
                return Err(unsupported(
                    "forced return is not available after a runtime-error stop",
                    "Clear the failure by restarting the debug session; this command is not exception recovery.",
                ));
            }
            self.require_stopped("frame.return")?;
            let task_id = self.task_for_frame(Some(frame_id))?;
            let frame = self
                .inspection_for_item(frame_id)?
                .stack(0, self.inspection_limits.max_frames)?
                .items
                .into_iter()
                .find(|frame| frame.id == frame_id)
                .ok_or_else(|| unknown_frame(frame_id))?;
            let function = {
                let worker = self
                    .runtime
                    .worker(task_id)
                    .ok_or_else(|| unknown_task(task_id))?;
                require_eligible(
                    self.state,
                    self.last_stop.reason,
                    self.last_stop.task_id,
                    &frame,
                    task_id,
                    self.runtime.task_state(task_id),
                    worker,
                )?;
                worker.function
            };
            let (convention, result_type) = {
                let info = self
                    .executable
                    .executable()
                    .functions
                    .get(usize::from(function.get()))
                    .ok_or_else(|| {
                        unsupported(
                            "forced return is not available because the active function metadata is missing",
                            "Rebuild the executable with the current compiler and retry.",
                        )
                    })?;
                (info.return_convention, info.debug.result_type)
            };
            let result_type = require_result_type(result_type)?;
            require_convention(convention, expression)?;
            reject_declared_category(&self.executable, result_type)?;
            let value = match expression {
                Some(expression) => {
                    self.evaluate_runtime_value(expression, Some(frame_id), limits)?
                }
                None => fpas_bytecode::Value::Unit,
            };
            prepare_return_value(&self.executable, result_type, &value, limits.max_depth)?;
            let prepared = self
                .inspections
                .get(&task_id)
                .ok_or_else(|| unknown_task(task_id))?
                .prepare_evaluation_result(&value, limits)?;
            let reserved_handles = prepared.reserved_handles();
            {
                let worker = self
                    .runtime
                    .worker_mut(task_id)
                    .ok_or_else(|| unknown_task(task_id))?;
                commit(worker, value.clone())?;
            }
            self.last_stop = {
                let Some(worker) = self.runtime.worker(task_id) else {
                    unreachable!("forced-return commit retains its selected worker")
                };
                stop_at_worker(
                    &self.executable,
                    worker,
                    DebugStopReason::Pause,
                    Vec::new(),
                    None,
                )
            };
            self.invalidate_inspection();
            self.refresh_inspection_with_reserved_handles(Some((task_id, reserved_handles)));
            self.inspection_task_id = task_id;
            let Some(inspection) = self.inspections.get_mut(&task_id) else {
                unreachable!("forced-return refresh retains its selected task snapshot")
            };
            let rendered = inspection.retain_prepared_evaluation_result(prepared);
            let Some(caller) = inspection
                .stack(0, 1)
                .ok()
                .and_then(|stack| stack.items.into_iter().next())
            else {
                unreachable!("an eligible forced return always restores one inspectable caller")
            };
            Ok(DebugForcedReturnResult {
                task_id,
                value: rendered.value,
                type_name: rendered.type_name,
                variables_reference: rendered.variables_reference,
                named_variables: rendered.named_variables,
                indexed_variables: rendered.indexed_variables,
                frame: caller,
            })
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_instruction_count(&self) -> u64 {
        self.runtime.instruction_count()
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_worker_registers(
        &self,
        task_id: u64,
    ) -> Option<(
        u16,
        usize,
        usize,
        usize,
        Vec<fpas_bytecode::Value>,
        Vec<bool>,
    )> {
        let worker = self.runtime.worker(task_id)?;
        Some((
            worker.function.get(),
            worker.ip,
            worker.base,
            worker.call_stack.len(),
            worker.registers[..worker.active_register_count].to_vec(),
            worker.register_initialized[..worker.active_register_count].to_vec(),
        ))
    }
}
