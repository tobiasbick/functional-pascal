//! Cooperative deadlines and cancellation for non-consuming completion barriers.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use super::super::{TaskSuspension, pool};
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{Register, TaskIntrinsic, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);
const CANCELLED: &str = "Task wait was cancelled";
const TIMED_OUT: &str = "Task wait timed out";

#[cfg(test)]
mod tests;

impl Worker {
    /// Wait for a completion while observing a relative timeout or cancellation token.
    pub(in crate::vm::tasks) fn controlled_wait_any(
        &mut self,
        operation: TaskIntrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let [tasks, control] = arguments else {
            return Err(
                self.task_type_error("task array and timeout or cancellation token", &Value::Unit)
            );
        };
        let ids = self.wait_any_ids(tasks)?;
        let (timeout, token) = match operation {
            TaskIntrinsic::WaitAnyWithTimeout => (Some(self.wait_timeout(control)?), None),
            TaskIntrinsic::WaitAnyWithCancellation => {
                let token = self.cancellation_handle(control, "CancellationToken")?;
                self.wait_token_cancelled(Some(token))?;
                (None, Some(token))
            }
            _ => unreachable!("controlled task wait dispatch"),
        };
        let started = Instant::now();
        let debug_deadline = if self.debug_tasks {
            timeout.map(|duration| {
                self.debug_clock_ref()
                    .now_millis()
                    .saturating_add(duration.as_millis() as u64)
            })
        } else {
            None
        };
        let mut initial = true;
        loop {
            let expired = if self.debug_tasks {
                debug_deadline
                    .is_some_and(|deadline| self.debug_clock_ref().now_millis() >= deadline)
            } else {
                timeout.is_some_and(|timeout| started.elapsed() >= timeout)
            };
            if let Some(value) = self.controlled_wait_result(&ids, token, expired, initial)? {
                return Ok(Some(Some(value)));
            }
            initial = false;
            if self.debug_tasks {
                self.task_suspension = Some(TaskSuspension::WaitAnyControlled {
                    ids,
                    token,
                    deadline_millis: debug_deadline,
                    destination,
                });
                self.suspend_requested = true;
                return Ok(Some(None));
            }
            let scheduler = Arc::clone(self.scheduler_ref()?);
            scheduler.fail_pending_batch_if_shutdown(&ids);
            if scheduler.is_shutdown() {
                continue;
            }
            if let Some(task) = scheduler.try_dequeue() {
                pool::run_helped(self, task, Arc::clone(&scheduler))?;
            } else {
                let interval = timeout.map_or(POLL_INTERVAL, |timeout| {
                    timeout.saturating_sub(started.elapsed()).min(POLL_INTERVAL)
                });
                scheduler.wait_for_any_change(&ids, Some(interval));
            }
        }
    }

    fn wait_token_cancelled(&self, token: Option<u64>) -> Result<bool, VmError> {
        token.map_or(Ok(false), |token| {
            self.hosted
                .cancellations
                .is_cancelled(token)
                .map_err(|message| self.cancellation_error(message))
        })
    }

    fn controlled_wait_result(
        &self,
        ids: &[u64],
        token: Option<u64>,
        expired: bool,
        initial: bool,
    ) -> Result<Option<Value>, VmError> {
        // Validate every identity and preserve task failures before considering control outcomes.
        let result = self.wait_any_result(ids)?;
        if self.wait_token_cancelled(token)? {
            return Ok(Some(control_error(CANCELLED)));
        }
        if expired && !initial {
            return Ok(Some(control_error(TIMED_OUT)));
        }
        if let Some(value) = result {
            return Ok(Some(Value::result_ok(value)));
        }
        Ok(expired.then(|| control_error(TIMED_OUT)))
    }

    /// Poll a controlled debugger wait without resetting its clock deadline.
    pub(in crate::vm::tasks) fn poll_debug_controlled_wait_any(
        &mut self,
        ids: Vec<u64>,
        token: Option<u64>,
        deadline_millis: Option<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        let expired =
            deadline_millis.is_some_and(|deadline| self.debug_clock_ref().now_millis() >= deadline);
        if let Some(value) = self.controlled_wait_result(&ids, token, expired, false)? {
            if let Some(destination) = destination {
                self.write(destination, value)?;
            }
            Ok(true)
        } else {
            self.task_suspension = Some(TaskSuspension::WaitAnyControlled {
                ids,
                token,
                deadline_millis,
                destination,
            });
            Ok(false)
        }
    }
}

fn control_error(message: &str) -> Value {
    Value::result_error(Value::Str(message.into()))
}
