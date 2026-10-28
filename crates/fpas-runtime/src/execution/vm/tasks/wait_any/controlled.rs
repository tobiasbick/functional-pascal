//! Cooperative deadlines and cancellation for non-consuming completion barriers.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use super::super::TaskSuspension;
use crate::execution::vm::shared::wakeups::WakeSignal;
use crate::execution::vm::tasks::errors::TaskErrorKind;
use crate::execution::vm::{VmError, worker::Worker};
use crate::{Register, TaskIntrinsic, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[cfg(test)]
mod tests;

impl Worker {
    /// Wait for a completion while observing a relative timeout or cancellation token.
    pub(in crate::execution::vm::tasks) fn controlled_wait_any(
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
        if let Value::Array(values) = tasks
            && let Err(reason) = super::validate_task_count(values.len())
        {
            return Ok(Some(Some(self.task_input_error("Tasks", reason)?)));
        }
        let ids = self.wait_any_ids(tasks)?;
        if operation == TaskIntrinsic::WaitAnyWithTimeout
            && matches!(control, Value::Integer(value) if *value < 0)
        {
            return Ok(Some(Some(self.task_input_error(
                "TimeoutMillis",
                "Timeout must be non-negative",
            )?)));
        }

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
        let suspended_deadline = if self.debug_tasks || self.task_id != 0 {
            timeout.map(|duration| self.task_clock_ref().deadline_after(duration))
        } else {
            None
        };
        loop {
            // Main-task waits register before probing. Suspended children use the shared clock
            // and timer driver instead of keeping the caller's stack while helping other tasks.
            let wake = if self.debug_tasks || self.task_id != 0 {
                None
            } else {
                let signal = WakeSignal::new();
                let scheduler_registration = self.scheduler_ref()?.subscribe(&signal);
                let cancellation_registration = token
                    .map(|token| self.hosted.cancellations.subscribe(token, &signal))
                    .transpose()
                    .map_err(|message| self.cancellation_error(message))?;
                Some((signal, scheduler_registration, cancellation_registration))
            };
            let expired = if self.debug_tasks || self.task_id != 0 {
                suspended_deadline
                    .is_some_and(|deadline| self.task_clock_ref().now_millis() >= deadline)
            } else {
                timeout.is_some_and(|timeout| started.elapsed() >= timeout)
            };
            if let Some(value) = self.controlled_wait_result(&ids, token, expired)? {
                return Ok(Some(Some(value)));
            }
            if self.debug_tasks || self.task_id != 0 {
                self.task_suspension = Some(TaskSuspension::WaitAnyControlled {
                    ids,
                    token,
                    deadline_millis: suspended_deadline,
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
            self.check_task_cancellation()?;
            let interval = timeout.map_or(POLL_INTERVAL, |timeout| {
                timeout.saturating_sub(started.elapsed()).min(POLL_INTERVAL)
            });
            if let Some((signal, _, _)) = &wake {
                signal.wait(interval);
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
    ) -> Result<Option<Value>, VmError> {
        // Validate every identity and preserve task failures before considering control outcomes.
        let result = self.wait_any_result(ids)?;
        if self.wait_token_cancelled(token)? {
            return Ok(Some(self.task_outcome_error(
                TaskErrorKind::Cancelled,
                "Std.Tasks.WaitAnyWithCancellation",
            )?));
        }
        if let Some(value) = result {
            return Ok(Some(Value::result_ok(value)));
        }
        if expired {
            Ok(Some(self.task_outcome_error(
                TaskErrorKind::Timeout,
                "Std.Tasks.WaitAnyWithTimeout",
            )?))
        } else {
            Ok(None)
        }
    }

    /// Poll a controlled task wait without resetting its clock deadline.
    pub(in crate::execution::vm::tasks) fn poll_controlled_wait_any(
        &mut self,
        ids: Vec<u64>,
        token: Option<u64>,
        deadline_millis: Option<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        let expired =
            deadline_millis.is_some_and(|deadline| self.task_clock_ref().now_millis() >= deadline);
        if let Some(value) = self.controlled_wait_result(&ids, token, expired)? {
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
