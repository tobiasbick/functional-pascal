//! Stopped-state orchestration for one protocol-neutral forced return.

use std::sync::atomic::Ordering;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::forced_return::{
    DebugForcedReturnResult, EligibilityContext, PreparedEntryCompletion, commit, prepare_entry,
    prepare_return_value, prepare_selection, reject_declared_category, require_convention,
    require_eligible, require_result_type, unknown_frame, unsupported,
};

#[cfg(test)]
type TestWorkerRegisters = (
    u16,
    usize,
    usize,
    usize,
    Vec<fpas_bytecode::Value>,
    Vec<bool>,
);

impl DebugSession {
    /// Complete a selected ordinary callee with a validated return value and remain stopped in its caller.
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

    /// Complete a selected callee using explicit evaluation and validation limits.
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
            let recovery = if self.state == DebugSessionState::Failed
                && self.last_stop.reason == DebugStopReason::RuntimeError
            {
                Some(self.last_stop.diagnostic.clone().ok_or_else(|| {
                    unsupported(
                        "forced return cannot recover a runtime-error stop without its exact diagnostic",
                        "Restart the debug session because the stopped failure identity is incomplete.",
                    )
                })?)
            } else {
                None
            };
            if recovery.is_some() {
                self.require_inspectable("frame.return")?;
            } else {
                self.require_stopped("frame.return")?;
            }
            let task_id = self.task_for_frame(Some(frame_id))?;
            let frame = self
                .inspection_for_item(frame_id)?
                .stack(0, self.inspection_limits.max_frames)?
                .items
                .into_iter()
                .find(|frame| frame.id == frame_id)
                .ok_or_else(|| unknown_frame(frame_id))?;
            enum Target {
                Callee(crate::vm::debug::forced_return::PreparedSelection),
                Entry(PreparedEntryCompletion),
            }
            let prepared = {
                let worker = self
                    .runtime
                    .worker(task_id)
                    .ok_or_else(|| unknown_task(task_id))?;
                require_eligible(
                    EligibilityContext {
                        state: self.state,
                        stop_reason: self.last_stop.reason,
                        stop_task_id: self.last_stop.task_id,
                        frame_task_id: task_id,
                        task_state: self.runtime.task_state(task_id),
                        runtime_recovery: recovery.is_some(),
                    },
                    &frame,
                    worker,
                )?;
                if frame.depth == worker.call_stack.len() {
                    Target::Entry(prepare_entry(worker, task_id, frame.depth)?)
                } else {
                    Target::Callee(prepare_selection(worker, frame.depth)?)
                }
            };
            let selected_function = match &prepared {
                Target::Callee(prepared) => prepared.selected_function,
                Target::Entry(prepared) => prepared.function,
            };
            let (convention, result_type) = {
                let info = self
                    .executable
                    .executable()
                    .functions
                    .get(usize::from(selected_function.get()))
                    .ok_or_else(|| {
                        unsupported(
                            "forced return is not available because the selected function metadata is missing",
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
            let prepared_result = self
                .inspections
                .get(&task_id)
                .ok_or_else(|| unknown_task(task_id))?
                .prepare_evaluation_result(&value, limits)?;
            let reserved_handles = prepared_result.reserved_handles();
            if let Target::Entry(prepared) = prepared {
                let root = match recovery.as_ref() {
                    Some(diagnostic) => self
                        .runtime
                        .recover_failed_entry(diagnostic, prepared, value),
                    None => self.runtime.complete_entry(
                        prepared.task_id,
                        prepared.function,
                        prepared.base,
                        prepared.call_stack_len,
                        value,
                    ),
                }
                .ok_or_else(|| {
                    unsupported(
                        "forced entry completion cannot commit because the selected task changed",
                        "Request stack frames again for the current stop and retry.",
                    )
                })?;
                self.invalidate_inspection();
                let rendered = if root {
                    self.state = DebugSessionState::Terminated;
                    prepared_result.into_terminal_result()
                } else {
                    if recovery.is_some() {
                        self.state = DebugSessionState::Stopped;
                    }
                    let next_task = self
                        .runtime
                        .inspectable_task_ids()
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| {
                            unreachable!("a stopped child entry retains its root task")
                        });
                    let Some(worker) = self.runtime.worker(next_task) else {
                        unreachable!("the selected inspectable task retains its worker")
                    };
                    self.last_stop = stop_at_worker(
                        &self.executable,
                        worker,
                        DebugStopReason::Pause,
                        Vec::new(),
                        None,
                    );
                    self.last_stop.task_id = next_task;
                    self.refresh_inspection_with_reserved_handles(Some((
                        next_task,
                        reserved_handles,
                    )));
                    self.inspection_task_id = next_task;
                    let Some(inspection) = self.inspections.get_mut(&next_task) else {
                        unreachable!("inspection refresh retains the selected fallback task")
                    };
                    inspection.retain_prepared_evaluation_result(prepared_result)
                };
                return Ok(DebugForcedReturnResult {
                    task_id,
                    value: rendered.value,
                    type_name: rendered.type_name,
                    variables_reference: rendered.variables_reference,
                    named_variables: rendered.named_variables,
                    indexed_variables: rendered.indexed_variables,
                    unwound_frames: frame.depth.saturating_add(1),
                    frame: None,
                    terminated: root,
                });
            }
            let Target::Callee(prepared) = prepared else {
                unreachable!("entry completion returned above")
            };
            if let Some(diagnostic) = recovery.as_ref() {
                if !self.runtime.recover_failed_return(
                    task_id,
                    diagnostic,
                    &prepared,
                    value.clone(),
                ) {
                    return Err(unsupported(
                        "forced return cannot recover because the stopped failure changed",
                        "Request the current stop and stack again, then retry the exact failed frame.",
                    ));
                }
                self.state = DebugSessionState::Stopped;
            } else {
                let worker = self
                    .runtime
                    .worker_mut(task_id)
                    .ok_or_else(|| unknown_task(task_id))?;
                commit(worker, &prepared, value.clone())?;
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
            let rendered = inspection.retain_prepared_evaluation_result(prepared_result);
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
                unwound_frames: prepared.unwind_count,
                frame: Some(caller),
                terminated: false,
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
    ) -> Option<TestWorkerRegisters> {
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

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_poll_task_result(
        &self,
        task_id: u64,
    ) -> crate::vm::TaskResultPoll {
        self.runtime.test_poll_task_result(task_id)
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_enqueue_pending_task(
        &self,
        function: fpas_bytecode::FunctionId,
    ) {
        self.runtime.test_enqueue_pending_task(function);
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_advance_clock(&self, duration: std::time::Duration) {
        self.runtime.wait(duration);
    }
}
