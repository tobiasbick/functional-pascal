//! Attempt admission, result interception, and non-recursive retry scheduling.

use super::Deadline;
use crate::vm::tasks::{TaskState, TaskSuspension};
use crate::vm::{VmError, dispatch::DispatchStep, worker::Worker};
use fpas_bytecode::Value;
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_TASK_CANCELLED, RUNTIME_VM_SHUTDOWN};
use std::time::{Duration, Instant};

impl Worker {
    /// Admit an initial or retried attempt, checking cancellation before its first instruction.
    pub(in crate::vm) fn supervised_ready(&mut self) -> Result<bool, VmError> {
        let Some(supervision) = self.supervision.as_ref() else {
            return Ok(true);
        };
        if !supervision.admission_pending {
            return Ok(true);
        }
        self.check_supervisor_cancellation(supervision.token)?;
        let remaining = match supervision.deadline.as_ref() {
            None => Duration::ZERO,
            Some(Deadline::Realtime(deadline)) => {
                deadline.saturating_duration_since(Instant::now())
            }
            Some(Deadline::Debug(deadline)) => {
                Duration::from_millis(deadline.saturating_sub(self.task_clock_ref().now_millis()))
            }
        };
        if !remaining.is_zero() {
            self.park_supervised_retry(remaining)?;
            return Ok(false);
        }
        if let Some(supervision) = self.supervision.as_mut() {
            supervision.admission_pending = false;
            supervision.deadline = None;
        }
        Ok(true)
    }

    /// Expose only final outcomes; retryable failures restore the original entry and suspend.
    pub(in crate::vm) fn supervised_outcome(
        &mut self,
        outcome: Result<Value, VmError>,
    ) -> Result<Option<Value>, VmError> {
        let Some(mut supervision) = self.supervision.take() else {
            return outcome.map(Some);
        };
        let retryable = match &outcome {
            Ok(Value::ResultError(_)) => true,
            Err(error) => error.code == RUNTIME_PROGRAM_PANIC,
            _ => false,
        };
        if !retryable || supervision.policy.remaining == 0 {
            return outcome.map(Some);
        }
        self.check_supervisor_cancellation(supervision.token)?;
        supervision.policy.remaining -= 1;
        supervision.admission_pending = true;
        let delay = Duration::from_millis(supervision.policy.backoff_millis);
        supervision.deadline = Some(if self.debug_tasks {
            Deadline::Debug(
                self.task_clock_ref()
                    .now_millis()
                    .saturating_add(supervision.policy.backoff_millis),
            )
        } else {
            Deadline::Realtime(Instant::now() + delay)
        });
        let info = &self.executable.executable().functions
            [usize::from(supervision.function.function.get())];
        let mut task = TaskState::entry(
            self.task_id,
            &supervision.function,
            info,
            [Value::OpaqueHandle(supervision.token)],
            true,
        );
        task.instruction_count = self
            .instruction_count
            .saturating_add(self.callback_instruction_count.get());
        task.supervision = Some(supervision);
        *self = self.worker_for_task(task);
        self.park_supervised_retry(delay)?;
        Ok(None)
    }

    /// Apply identical attempt boundaries to single-instruction debugger dispatch.
    pub(in crate::vm) fn dispatch_supervised_debug_one(&mut self) -> Result<DispatchStep, VmError> {
        if !self.supervised_ready()? {
            return Ok(DispatchStep::Suspend);
        }
        let outcome = match self.dispatch_debug_one() {
            Ok(DispatchStep::Return(value)) => Ok(value),
            Err(error) => Err(error),
            other => return other,
        };
        self.supervised_outcome(outcome)
            .map(|value| value.map_or(DispatchStep::Suspend, DispatchStep::Return))
    }

    fn check_supervisor_cancellation(&self, token: u64) -> Result<(), VmError> {
        let scheduler = self.scheduler_ref()?;
        if scheduler.is_shutdown() {
            return Err(scheduler.first_error().unwrap_or_else(|| {
                self.runtime_error(
                    RUNTIME_VM_SHUTDOWN,
                    "Supervised task interrupted by VM shutdown",
                    "Keep the VM alive until required task groups have closed.",
                )
            }));
        }
        if self
            .hosted
            .cancellations
            .is_cancelled(token)
            .map_err(|message| self.group_error(message))?
        {
            return Err(self.runtime_error(
                RUNTIME_TASK_CANCELLED,
                "Supervised task was cancelled before its next attempt",
                "Use a new active group to start more work.",
            ));
        }
        Ok(())
    }

    fn park_supervised_retry(&mut self, remaining: Duration) -> Result<(), VmError> {
        if self.debug_tasks {
            let deadline_millis = match self
                .supervision
                .as_ref()
                .and_then(|state| state.deadline.as_ref())
            {
                Some(Deadline::Debug(deadline)) => *deadline,
                _ => self.task_clock_ref().now_millis(),
            };
            self.task_suspension = Some(TaskSuspension::SupervisionBackoff { deadline_millis });
        } else {
            let scheduler = std::sync::Arc::clone(self.scheduler_ref()?);
            let state = self.take_task_state();
            // Reuse the timer driver; bounded slices observe cancellation during long backoff.
            if remaining.is_zero() {
                scheduler.enqueue(state);
            } else {
                scheduler.schedule(state, remaining.as_millis().clamp(1, 10) as u64);
            }
        }
        self.suspend_requested = true;
        Ok(())
    }
}
