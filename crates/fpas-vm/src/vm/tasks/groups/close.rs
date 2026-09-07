//! Cooperative group joining with optional waiting budgets and retained timeout ownership.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md#closetaskgroupwithtimeout`.

use crate::vm::shared::wakeups::WakeSignal;
use crate::vm::tasks::{TaskSuspension, pool};
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{Register, Value};
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod tests;

impl Worker {
    /// Seal admission and cancel once, retaining all group ownership until a successful join.
    pub(super) fn start_group_close(
        &mut self,
        id: u64,
        timeout: Option<Duration>,
        destination: Option<Register>,
    ) -> Result<Option<Value>, VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        let deadline_millis =
            timeout.map(|duration| self.task_clock_ref().deadline_after(duration));
        scheduler
            .groups
            .begin_close(id, self.task_id)
            .map_err(|e| self.group_error(e))?;
        loop {
            let signal = WakeSignal::new();
            let registration = scheduler.subscribe(&signal);
            if let Some(value) = self.group_close_result(id, deadline_millis)? {
                return Ok(Some(value));
            }
            if self.debug_tasks || self.task_id != 0 {
                self.task_suspension = Some(TaskSuspension::GroupClose {
                    id,
                    deadline_millis,
                    destination,
                });
                self.suspend_requested = true;
                return Ok(None);
            }
            // A timed root wait must not enter arbitrary child code on its own stack: a
            // non-cooperative child could otherwise prevent the timeout from being observed.
            if deadline_millis.is_none()
                && let Some(task) = scheduler.try_dequeue()
            {
                drop(registration);
                pool::run_helped(self, task, Arc::clone(&scheduler))?;
            } else {
                let milliseconds = deadline_millis.map_or(10, |deadline| {
                    deadline
                        .saturating_sub(self.task_clock_ref().now_millis())
                        .min(10)
                });
                signal.wait(Duration::from_millis(milliseconds));
            }
        }
    }

    fn group_close_result(
        &self,
        id: u64,
        deadline_millis: Option<u64>,
    ) -> Result<Option<Value>, VmError> {
        // Real completion observed at this probe wins over expiry. Shutdown is checked by the
        // scheduler before releasing anything; synthetic completion cannot certify a join.
        if let Some(failures) = self.scheduler_ref()?.poll_group_close(id)? {
            let report = self.group_report(failures)?;
            return Ok(Some(if deadline_millis.is_some() {
                Value::result_ok(report)
            } else {
                report
            }));
        }
        Ok(deadline_millis
            .filter(|deadline| self.task_clock_ref().now_millis() >= *deadline)
            .map(|_| Value::result_error(Value::Str("Task group close timed out".into()))))
    }

    /// Resume with the original deadline; timeout never takes the group's children or reports.
    pub(in crate::vm::tasks) fn poll_group_close(
        &mut self,
        id: u64,
        deadline_millis: Option<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        if let Some(value) = self.group_close_result(id, deadline_millis)? {
            if let Some(destination) = destination {
                self.write(destination, value)?;
            }
            Ok(true)
        } else {
            self.task_suspension = Some(TaskSuspension::GroupClose {
                id,
                deadline_millis,
                destination,
            });
            Ok(false)
        }
    }
}
