//! Register-VM task opcodes and task-aware intrinsics.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`,
//! `docs/pascal/language/concurrency/scheduling.md`.

mod cancellation;
mod channel;
pub(in crate::vm) mod groups;
pub(super) mod pool;
mod scheduler;
pub(in crate::vm) mod selection;
mod spawn;
mod state;
pub(in crate::vm) mod supervision;
mod suspension;
mod timeouts;
mod wait_any;

pub(super) use scheduler::{RetainedResultReplacement, TaskScheduler};
pub(super) use state::TaskState;
pub(in crate::vm) use suspension::{TaskClock, TaskSuspension, TaskSuspensionState};

use std::sync::Arc;

use fpas_bytecode::{Intrinsic, Register, TaskIntrinsic, TimeIntrinsic, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

use super::worker::Worker;
use super::{VmError, diagnostics};
use crate::vm::{TaskBatchPoll, TaskResultPoll};

impl Worker {
    pub(super) fn yield_task(&mut self) {
        if self.debug_tasks {
            self.task_suspension = Some(TaskSuspension::Yield);
            self.suspend_requested = true;
        } else if self.task_id == 0 {
            std::thread::yield_now();
        } else {
            self.suspend_and_enqueue();
        }
    }

    /// Dispatch task-aware operations, preserving debugger suspension when enabled.
    pub(super) fn task_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let result = self.task_intrinsic_inner(intrinsic, arguments, destination)?;
        if !self.debug_tasks && self.task_id != 0 && self.task_suspension.is_some() {
            self.park_pool_suspension()?;
        }
        Ok(result)
    }

    fn task_intrinsic_inner(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        if let Some(value) = self.group_intrinsic(intrinsic, arguments, destination)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.selection_intrinsic(intrinsic, arguments, destination)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.cancellation_intrinsic(intrinsic, arguments)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.channel_intrinsic(intrinsic, arguments, destination)? {
            return Ok(Some(value));
        }
        if self.debug_tasks || (self.task_id != 0 && matches!(intrinsic, Intrinsic::Task(_))) {
            return self.cooperative_task_intrinsic(intrinsic, arguments, destination);
        }
        match intrinsic {
            Intrinsic::Task(TaskIntrinsic::WaitAny) => self.wait_any(arguments, destination),
            Intrinsic::Task(
                operation @ (TaskIntrinsic::WaitAnyWithTimeout
                | TaskIntrinsic::WaitAnyWithCancellation),
            ) => self.controlled_wait_any(operation, arguments, destination),
            Intrinsic::Task(TaskIntrinsic::Wait) => {
                let [Value::Task(id)] = arguments else {
                    return Err(
                        self.task_type_error("task", arguments.first().unwrap_or(&Value::Unit))
                    );
                };
                loop {
                    match self.scheduler_ref()?.poll_result(*id) {
                        TaskResultPoll::Available(value) => return Ok(Some(Some(value))),
                        TaskResultPoll::Failed(error) => return Err(error),
                        TaskResultPoll::Consumed => return Err(self.invalid_task(*id)),
                        TaskResultPoll::Unknown => return Err(self.invalid_task(*id)),
                        TaskResultPoll::Pending => {
                            let scheduler = self.scheduler_ref()?;
                            scheduler.fail_pending_result_if_shutdown(*id);
                            if !scheduler.is_shutdown() {
                                self.help_or_wait_result(*id)?;
                            }
                        }
                    }
                }
            }
            Intrinsic::Task(TaskIntrinsic::WaitAll) => {
                let [Value::Array(values)] = arguments else {
                    return Err(
                        self.task_type_error("array", arguments.first().unwrap_or(&Value::Unit))
                    );
                };
                let mut ids = values
                    .iter()
                    .map(|value| match value {
                        Value::Task(id) => Ok(*id),
                        other => Err(self.task_type_error("task", other)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ids.sort_unstable();
                ids.dedup();
                loop {
                    match self.scheduler_ref()?.poll_batch(&ids) {
                        TaskBatchPoll::Complete => return Ok(Some(None)),
                        TaskBatchPoll::Failed(error) => return Err(error),
                        TaskBatchPoll::Unknown(id) => return Err(self.invalid_task(id)),
                        TaskBatchPoll::Pending => {
                            let scheduler = self.scheduler_ref()?;
                            scheduler.fail_pending_batch_if_shutdown(&ids);
                            if !scheduler.is_shutdown() {
                                self.help_or_wait_batch(&ids)?;
                            }
                        }
                    }
                }
            }
            Intrinsic::Time(TimeIntrinsic::Sleep) if self.task_id != 0 => {
                let [Value::Integer(milliseconds)] = arguments else {
                    return Err(
                        self.task_type_error("integer", arguments.first().unwrap_or(&Value::Unit))
                    );
                };
                let milliseconds = u64::try_from(*milliseconds).map_err(|_| {
                    self.task_type_error("non-negative integer", &Value::Integer(*milliseconds))
                })?;
                let state = self.take_task_state();
                self.scheduler_ref()?.schedule(state, milliseconds);
                self.suspend_requested = true;
                Ok(Some(None))
            }
            _ => Ok(None),
        }
    }

    fn cooperative_task_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        match intrinsic {
            Intrinsic::Task(TaskIntrinsic::WaitAny) => self.wait_any(arguments, destination),
            Intrinsic::Task(
                operation @ (TaskIntrinsic::WaitAnyWithTimeout
                | TaskIntrinsic::WaitAnyWithCancellation),
            ) => self.controlled_wait_any(operation, arguments, destination),
            Intrinsic::Task(TaskIntrinsic::Wait) => {
                let [Value::Task(id)] = arguments else {
                    return Err(
                        self.task_type_error("task", arguments.first().unwrap_or(&Value::Unit))
                    );
                };
                match self.scheduler_ref()?.poll_result(*id) {
                    TaskResultPoll::Available(value) => Ok(Some(Some(value))),
                    TaskResultPoll::Failed(error) => Err(error),
                    TaskResultPoll::Consumed | TaskResultPoll::Unknown => {
                        Err(self.invalid_task(*id))
                    }
                    TaskResultPoll::Pending => {
                        self.task_suspension = Some(TaskSuspension::Wait {
                            id: *id,
                            destination,
                        });
                        self.suspend_requested = true;
                        Ok(Some(None))
                    }
                }
            }
            Intrinsic::Task(TaskIntrinsic::WaitAll) => {
                let [Value::Array(values)] = arguments else {
                    return Err(
                        self.task_type_error("array", arguments.first().unwrap_or(&Value::Unit))
                    );
                };
                let mut ids = values
                    .iter()
                    .map(|value| match value {
                        Value::Task(id) => Ok(*id),
                        other => Err(self.task_type_error("task", other)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ids.sort_unstable();
                ids.dedup();
                match self.scheduler_ref()?.poll_batch(&ids) {
                    TaskBatchPoll::Complete => Ok(Some(None)),
                    TaskBatchPoll::Failed(error) => Err(error),
                    TaskBatchPoll::Unknown(id) => Err(self.invalid_task(id)),
                    TaskBatchPoll::Pending => {
                        self.task_suspension = Some(TaskSuspension::WaitAll { ids });
                        self.suspend_requested = true;
                        Ok(Some(None))
                    }
                }
            }
            Intrinsic::Time(TimeIntrinsic::Sleep) if self.task_id != 0 => {
                let [Value::Integer(milliseconds)] = arguments else {
                    return Err(
                        self.task_type_error("integer", arguments.first().unwrap_or(&Value::Unit))
                    );
                };
                let milliseconds = u64::try_from(*milliseconds).map_err(|_| {
                    self.task_type_error("non-negative integer", &Value::Integer(*milliseconds))
                })?;
                self.task_suspension =
                    Some(TaskSuspension::sleep(milliseconds, self.task_clock_ref()));
                self.suspend_requested = true;
                Ok(Some(None))
            }
            _ => Ok(None),
        }
    }

    /// Resume a pool or debugger task when its suspended operation becomes ready.
    pub(in crate::vm) fn poll_task_suspension(&mut self) -> Result<bool, VmError> {
        let Some(suspension) = self.task_suspension.take() else {
            self.suspend_requested = false;
            return Ok(true);
        };
        let ready = match suspension {
            TaskSuspension::SupervisionBackoff { .. } => self.supervised_ready(),
            TaskSuspension::GroupClose {
                id,
                deadline_millis,
                destination,
            } => self.poll_group_close(id, deadline_millis, destination),
            TaskSuspension::Selection(wait) => self.poll_selection(*wait),
            TaskSuspension::Yield => Ok(true),
            TaskSuspension::WaitAnyControlled {
                ids,
                token,
                deadline_millis,
                destination,
            } => self.poll_controlled_wait_any(ids, token, deadline_millis, destination),
            TaskSuspension::WaitAny { ids, destination } => self.poll_wait_any(ids, destination),
            TaskSuspension::Wait { id, destination } => {
                match self.scheduler_ref()?.poll_result(id) {
                    TaskResultPoll::Available(value) => {
                        if let Some(destination) = destination {
                            self.write(destination, value)?;
                        }
                        Ok(true)
                    }
                    TaskResultPoll::Failed(error) => Err(error),
                    TaskResultPoll::Consumed | TaskResultPoll::Unknown => {
                        Err(self.invalid_task(id))
                    }
                    TaskResultPoll::Pending => {
                        self.task_suspension = Some(TaskSuspension::Wait { id, destination });
                        Ok(false)
                    }
                }
            }
            TaskSuspension::WaitAll { ids } => match self.scheduler_ref()?.poll_batch(&ids) {
                TaskBatchPoll::Complete => Ok(true),
                TaskBatchPoll::Failed(error) => Err(error),
                TaskBatchPoll::Unknown(id) => Err(self.invalid_task(id)),
                TaskBatchPoll::Pending => {
                    self.task_suspension = Some(TaskSuspension::WaitAll { ids });
                    Ok(false)
                }
            },
            TaskSuspension::ChannelSend {
                handle,
                value,
                token,
                destination,
            } => self.poll_channel_send(handle, value, token, destination),
            TaskSuspension::ChannelReceive {
                handle,
                token,
                destination,
            } => self.poll_channel_receive(handle, token, destination),
            TaskSuspension::ChannelSendTimeout {
                handle,
                value,
                deadline_millis,
                destination,
            } => self.poll_channel_send_timeout(handle, value, deadline_millis, destination),
            TaskSuspension::ChannelReceiveTimeout {
                handle,
                deadline_millis,
                destination,
            } => self.poll_channel_receive_timeout(handle, deadline_millis, destination),
            TaskSuspension::Sleep { deadline_millis }
                if self.task_clock_ref().now_millis() >= deadline_millis =>
            {
                Ok(true)
            }
            TaskSuspension::Sleep { deadline_millis } => {
                self.task_suspension = Some(TaskSuspension::Sleep { deadline_millis });
                Ok(false)
            }
        }?;
        if ready {
            self.suspend_requested = false;
        }
        Ok(ready)
    }

    fn help_or_wait_result(&mut self, id: u64) -> Result<(), VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        if let Some(task) = scheduler.try_dequeue() {
            pool::run_helped(self, task, Arc::clone(&scheduler))?;
        } else {
            scheduler.wait_for_result(id);
        }
        Ok(())
    }
    fn help_or_wait_batch(&mut self, ids: &[u64]) -> Result<(), VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        if let Some(task) = scheduler.try_dequeue() {
            pool::run_helped(self, task, Arc::clone(&scheduler))?;
        } else {
            scheduler.wait_for_batch(ids);
        }
        Ok(())
    }
    /// Access the shared scheduler for task and hosted resource ownership.
    pub(in crate::vm) fn scheduler_ref(&self) -> Result<&Arc<TaskScheduler>, VmError> {
        self.scheduler.as_ref().ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Task intrinsic ran without a scheduler",
            )
        })
    }
    fn task_clock_ref(&self) -> &TaskClock {
        let Some(clock) = self.task_clock.as_deref() else {
            unreachable!("task suspension requires a monotonic task clock")
        };
        clock
    }
    fn task_type_error(&self, expected: &str, actual: &Value) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Expected {expected}, got `{}`", actual.type_name()),
            format!("Pass a {expected} value."),
        )
    }
    fn invalid_task(&self, id: u64) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_INVALID_TASK,
            format!("Task {id} was not created by this VM or its result was already consumed"),
            "Pass an unconsumed task handle returned by a `go` expression.",
        )
    }
}
