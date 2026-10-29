//! Ordered probing and exactly-one source commitment for a mixed selection.

use super::*;
use crate::execution::vm::channels::{ReceiveState, SendState};
use crate::execution::vm::shared::wakeups::{WakeRegistration, WakeSignal};
use crate::execution::vm::tasks::TaskSuspension;
use crate::execution::vm::{TaskAnyPoll, TaskBatchPoll};
use std::sync::Arc;

impl Worker {
    /// Validate every task identity and preserve original failure diagnostics.
    pub(super) fn validate_selection_tasks(&self, ids: &[u64]) -> Result<(), VmError> {
        match self.scheduler_ref()?.poll_batch(ids) {
            TaskBatchPoll::Unknown(id) => Err(self.invalid_task(id)),
            TaskBatchPoll::Failed { id, error } => Err(self.observe_task_failure(id, error)),
            _ => Ok(()),
        }
    }

    fn validate_selection_sources(&self, wait: &SelectionWait) -> Result<(), VmError> {
        let mut tasks = Vec::new();
        for case in &wait.cases {
            if let CaseSource::Send(_, value) = &case.source {
                self.require_transferable_values(std::slice::from_ref(value))?;
            }
            match case.source {
                CaseSource::Send(id, _) | CaseSource::Receive(id) => self
                    .hosted
                    .channels
                    .validate(id)
                    .map_err(|e| self.selection_error(e))?,
                CaseSource::Cancellation(id) => {
                    self.hosted
                        .cancellations
                        .is_cancelled(id)
                        .map_err(|e| self.selection_error(e))?;
                }
                CaseSource::Task(id) => tasks.push(id),
                CaseSource::Timer(_) | CaseSource::Process(_, _) => {}
            }
        }
        self.validate_selection_tasks(&tasks)
    }

    fn selection_registrations(
        &self,
        wait: &SelectionWait,
        signal: &Arc<WakeSignal>,
    ) -> Result<Vec<WakeRegistration>, VmError> {
        let mut registrations = vec![self.scheduler_ref()?.subscribe(signal)];
        for case in &wait.cases {
            let registration = match case.source {
                CaseSource::Send(id, _) => self.hosted.channels.subscribe_send(id, signal),
                CaseSource::Receive(id) => self.hosted.channels.subscribe_receive(id, signal),
                CaseSource::Cancellation(id) => self.hosted.cancellations.subscribe(id, signal),
                _ => continue,
            }
            .map_err(|e| self.selection_error(e))?;
            registrations.push(registration);
        }
        Ok(registrations)
    }

    fn selection_probe(
        &self,
        wait: &SelectionWait,
    ) -> Result<Option<(usize, Option<Value>)>, VmError> {
        self.validate_selection_sources(wait)?;
        let elapsed = if self.debug_tasks {
            Duration::from_millis(
                self.task_clock_ref()
                    .now_millis()
                    .saturating_sub(wait.debug_started),
            )
        } else {
            wait.started.elapsed()
        };
        for (index, case) in wait.cases.iter().enumerate() {
            let argument = match &case.source {
                CaseSource::Process(handle, operation) => {
                    match self.poll_process_case(*handle, operation)? {
                        Some(outcome) => Some(outcome),
                        None => continue,
                    }
                }
                CaseSource::Receive(id) => match self
                    .hosted
                    .channels
                    .receive(*id, false, None)
                    .map_err(|e| self.selection_error(e))?
                {
                    ReceiveState::Received(value) => Some(Value::result_ok(value)),
                    ReceiveState::Closed => Some(self.task_outcome_error(
                        crate::execution::vm::tasks::errors::TaskErrorKind::Closed,
                        "Std.Tasks.ReceiveCase",
                    )?),
                    _ => continue,
                },
                CaseSource::Send(id, value) => match self
                    .hosted
                    .channels
                    .send(*id, value.clone(), false, None)
                    .map_err(|e| self.selection_error(e))?
                {
                    SendState::Sent => Some(Value::result_ok(Value::Boolean(true))),
                    SendState::Closed => Some(self.task_outcome_error(
                        crate::execution::vm::tasks::errors::TaskErrorKind::Closed,
                        "Std.Tasks.SendCase",
                    )?),
                    _ => continue,
                },
                CaseSource::Task(id) => match self.scheduler_ref()?.poll_any(&[*id]) {
                    TaskAnyPoll::Complete(_) => None,
                    TaskAnyPoll::Failed { id, error } => {
                        return Err(self.observe_task_failure(id, error));
                    }
                    TaskAnyPoll::Unknown(id) => return Err(self.invalid_task(id)),
                    TaskAnyPoll::Pending => continue,
                },
                CaseSource::Timer(ms) if elapsed >= Duration::from_millis(*ms) => None,
                CaseSource::Cancellation(id)
                    if self
                        .hosted
                        .cancellations
                        .is_cancelled(*id)
                        .map_err(|e| self.selection_error(e))? =>
                {
                    None
                }
                _ => continue,
            };
            return Ok(Some((index, argument)));
        }
        Ok(None)
    }

    fn finish_selection(
        &mut self,
        mut wait: SelectionWait,
        index: usize,
        argument: Option<Value>,
    ) -> Result<(), VmError> {
        let winner = wait.cases.remove(index);
        let destination = wait.destination;
        drop(wait);
        self.start_selection_callback(winner.callback, argument, index, destination)
    }

    /// Wait or suspend until exactly one source operation commits.
    ///
    /// The root keeps probing events instead of running unrelated child work inline.
    /// See `docs/pascal/std/concurrency/task.md`.
    pub(super) fn run_selection(&mut self, wait: SelectionWait) -> Result<(), VmError> {
        if self.debug_tasks {
            self.poll_selection(wait)?;
            return Ok(());
        }
        loop {
            let signal = WakeSignal::new();
            let registrations = self.selection_registrations(&wait, &signal)?;
            if let Some((index, argument)) = self.selection_probe(&wait)? {
                drop(registrations);
                return self.finish_selection(wait, index, argument);
            }
            let scheduler = Arc::clone(self.scheduler_ref()?);
            if scheduler.is_shutdown() {
                return Err(scheduler
                    .first_error()
                    .unwrap_or_else(|| self.selection_error("Runtime shut down while selecting")));
            }
            if self.task_id != 0 {
                // A helped child must yield its stack so its waiting parent can make progress.
                drop(registrations);
                self.task_suspension = Some(TaskSuspension::Selection(Box::new(wait)));
                self.park_pool_suspension()?;
                return Ok(());
            }
            // Inline helping can hold an event-loop task inside arbitrary CPU work until
            // that work yields. Pool workers execute queued tasks while the root waits.
            let interval = wait
                .cases
                .iter()
                .filter_map(|case| match case.source {
                    CaseSource::Timer(ms) => {
                        Some(Duration::from_millis(ms).saturating_sub(wait.started.elapsed()))
                    }
                    _ => None,
                })
                .min()
                .unwrap_or(Duration::from_millis(10))
                .min(Duration::from_millis(10));
            self.check_task_cancellation()?;
            {
                // Claimed cases own registrations throughout this wait.
                let _wait = self.native_wait(Vec::new())?;
                signal.wait(interval);
            }
        }
    }

    /// Resume a cooperative selection without resetting its shared timer origin.
    pub(in crate::execution::vm::tasks) fn poll_selection(
        &mut self,
        wait: SelectionWait,
    ) -> Result<bool, VmError> {
        if let Some((index, argument)) = self.selection_probe(&wait)? {
            self.finish_selection(wait, index, argument)?;
            Ok(true)
        } else {
            self.task_suspension = Some(TaskSuspension::Selection(Box::new(wait)));
            self.suspend_requested = true;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests;
