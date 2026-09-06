//! Non-consuming task-completion barriers.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use super::{TaskSuspension, pool};
use crate::vm::{TaskAnyPoll, VmError, diagnostics, worker::Worker};
use fpas_bytecode::{Register, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;
use std::sync::Arc;

const MAX_TASKS: usize = 1_048_576;

fn validate_task_count(count: usize) -> Result<(), &'static str> {
    if (1..=MAX_TASKS).contains(&count) {
        Ok(())
    } else {
        Err("WaitAny requires between 1 and 1048576 task handles")
    }
}

#[cfg(test)]
mod tests;

impl Worker {
    /// Validate a bounded task list and wait without consuming any successful result.
    pub(super) fn wait_any(
        &mut self,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let [Value::Array(values)] = arguments else {
            return Err(
                self.task_type_error("array of task", arguments.first().unwrap_or(&Value::Unit))
            );
        };
        if let Err(message) = validate_task_count(values.len()) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_INVALID_TASK,
                message,
                "Pass a non-empty array of retained task handles.",
            ));
        }
        let ids = values
            .iter()
            .map(|value| match value {
                Value::Task(id) => Ok(*id),
                other => Err(self.task_type_error("task", other)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        loop {
            if let Some(value) = self.wait_any_result(&ids)? {
                return Ok(Some(Some(value)));
            }
            if self.debug_tasks {
                self.task_suspension = Some(TaskSuspension::WaitAny { ids, destination });
                self.suspend_requested = true;
                return Ok(Some(None));
            }
            let scheduler = Arc::clone(self.scheduler_ref()?);
            scheduler.fail_pending_batch_if_shutdown(&ids);
            if !scheduler.is_shutdown() {
                if let Some(task) = scheduler.try_dequeue() {
                    pool::run_helped(self, task, Arc::clone(&scheduler))?;
                } else {
                    scheduler.wait_for_any(&ids);
                }
            }
        }
    }

    fn wait_any_result(&self, ids: &[u64]) -> Result<Option<Value>, VmError> {
        match self.scheduler_ref()?.poll_any(ids) {
            TaskAnyPoll::Complete(index) => Ok(Some(Value::Integer(index as i64))),
            TaskAnyPoll::Pending => Ok(None),
            TaskAnyPoll::Unknown(id) => Err(self.invalid_task(id)),
            TaskAnyPoll::Failed(error) => Err(error),
        }
    }

    /// Resume a debugger-owned completion barrier or retain its suspension.
    pub(super) fn poll_debug_wait_any(
        &mut self,
        ids: Vec<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        if let Some(value) = self.wait_any_result(&ids)? {
            if let Some(destination) = destination {
                self.write(destination, value)?;
            }
            Ok(true)
        } else {
            self.task_suspension = Some(TaskSuspension::WaitAny { ids, destination });
            Ok(false)
        }
    }
}
