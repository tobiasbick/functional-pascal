//! Register-VM task opcodes and task-aware intrinsics.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`,
//! `docs/pascal/language/concurrency/scheduling.md`.

mod cancellation;
mod channel;
pub(super) mod pool;
mod scheduler;
mod state;
mod suspension;

pub(super) use scheduler::{RetainedResultReplacement, TaskScheduler};
pub(super) use state::TaskState;
pub(in crate::vm) use suspension::{DebugClock, TaskSuspension, TaskSuspensionState};

use std::sync::Arc;

use fpas_bytecode::{AbcOperands, Intrinsic, Register, TaskIntrinsic, TimeIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

use super::worker::Worker;
use super::{VmError, diagnostics};
use crate::vm::{TaskBatchPoll, TaskResultPoll};

impl Worker {
    pub(super) fn spawn_task(
        &mut self,
        operands: AbcOperands,
        detached: bool,
    ) -> Result<(), VmError> {
        let scheduler = self
            .scheduler
            .as_ref()
            .ok_or_else(|| {
                self.unavailable_opcode(if detached {
                    fpas_bytecode::Opcode::SpawnDetachedTask
                } else {
                    fpas_bytecode::Opcode::SpawnTask
                })
            })?
            .clone();
        let (callee_register, argument_base) = if detached {
            (operands.a, operands.b)
        } else {
            (operands.b, operands.c)
        };
        let callee = self
            .read(Register::new(callee_register).map_err(|e| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    e.to_string(),
                )
            })?)?
            .clone();
        let Value::Function(function) = callee else {
            return Err(self.task_type_error("function", &callee));
        };
        if function.task_bound {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_INVALID_TASK,
                format!(
                    "Cannot spawn task-bound closure `{}` across a task boundary",
                    function.name
                ),
                "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.",
            ));
        }
        let target = function.function;
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Task target is outside the function table",
                )
            })?;
        let visible_arity = usize::from(info.arity)
            .checked_sub(usize::from(function.bound_receiver.is_some()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Bound task target has no receiver parameter",
                )
            })?;
        if visible_arity != usize::from(operands.auxiliary) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {} arguments, got {}",
                    function.name, visible_arity, operands.auxiliary
                ),
                "Spawn the task with the declared number of arguments.",
            ));
        }
        let register_count = info.register_count;
        let task_start = info.code.start;
        let arguments = self.clone_window(argument_base, operands.auxiliary)?;
        let (registers, register_initialized) = Self::register_window(
            usize::from(register_count),
            function
                .bound_receiver
                .iter()
                .cloned()
                .chain(arguments)
                .chain(function.captures.iter().cloned()),
        );
        let id = scheduler.alloc_id();
        if !detached {
            scheduler.register_result(id);
            self.write(
                Register::new(operands.a).map_err(|e| {
                    diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        e.to_string(),
                    )
                })?,
                Value::Task(id),
            )?;
        }
        scheduler.enqueue(TaskState {
            id,
            function: target,
            ip: usize::try_from(task_start.get()).map_err(|_| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Task address does not fit this host",
                )
            })?,
            base: 0,
            registers,
            register_initialized,
            frames: Vec::new(),
            retain_result: !detached,
            instruction_count: 0,
            suppressed_initializers: Vec::new(),
            callback_continuations: Vec::new(),
        });
        Ok(())
    }

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

    pub(super) fn task_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        if let Some(value) = self.cancellation_intrinsic(intrinsic, arguments)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.channel_intrinsic(intrinsic, arguments, destination)? {
            return Ok(Some(value));
        }
        if self.debug_tasks {
            return self.debug_task_intrinsic(intrinsic, arguments, destination);
        }
        match intrinsic {
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

    fn debug_task_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        match intrinsic {
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
                    Some(TaskSuspension::sleep(milliseconds, self.debug_clock_ref()));
                self.suspend_requested = true;
                Ok(Some(None))
            }
            _ => Ok(None),
        }
    }

    pub(in crate::vm) fn poll_debug_suspension(&mut self) -> Result<bool, VmError> {
        let Some(suspension) = self.task_suspension.take() else {
            self.suspend_requested = false;
            return Ok(true);
        };
        let ready = match suspension {
            TaskSuspension::Yield => Ok(true),
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
            } => self.poll_debug_channel_send(handle, value, token, destination),
            TaskSuspension::ChannelReceive {
                handle,
                token,
                destination,
            } => self.poll_debug_channel_receive(handle, token, destination),
            TaskSuspension::ChannelSendTimeout {
                handle,
                value,
                deadline_millis,
                destination,
            } => self.poll_debug_channel_send_timeout(handle, value, deadline_millis, destination),
            TaskSuspension::ChannelReceiveTimeout {
                handle,
                deadline_millis,
                destination,
            } => self.poll_debug_channel_receive_timeout(handle, deadline_millis, destination),
            TaskSuspension::Sleep { deadline_millis }
                if self.debug_clock_ref().now_millis() >= deadline_millis =>
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
    fn scheduler_ref(&self) -> Result<&Arc<TaskScheduler>, VmError> {
        self.scheduler.as_ref().ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Task intrinsic ran without a scheduler",
            )
        })
    }
    fn debug_clock_ref(&self) -> &DebugClock {
        let Some(clock) = self.debug_clock.as_deref() else {
            unreachable!("debug task suspension requires a debugger clock")
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
