//! Join every creating-task group before publishing its terminal result.
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

use crate::Value;
use crate::execution::vm::shared::wakeups::WakeSignal;
use crate::execution::vm::tasks::TaskSuspension;
use crate::execution::vm::{VmError, worker::Worker};
use std::sync::Arc;
use std::time::Duration;

impl Worker {
    /// Cancel open groups and retain the body's outcome until all children terminate.
    pub(in crate::execution::vm) fn finish_owned_groups(
        &mut self,
        outcome: Result<Value, VmError>,
    ) -> Result<Option<Value>, VmError> {
        let Some(scheduler) = self.scheduler.clone() else {
            return outcome.map(Some);
        };
        let groups = scheduler.groups.owned_groups(self.task_id);
        if groups.is_empty() {
            return outcome.map(Some);
        }
        for &group in &groups {
            scheduler
                .groups
                .begin_close(group, self.task_id)
                .map_err(|message| self.group_error(message))?;
        }
        if self.poll_owned_groups(groups, outcome)? {
            #[expect(
                clippy::expect_used,
                reason = "poll_owned_groups stores the terminal outcome before returning true"
            )]
            self.pending_completion
                .take()
                .expect("joined owners retain their outcome")
                .map(Some)
        } else {
            self.suspend_requested = true;
            Ok(None)
        }
    }

    /// Resume cleanup without allowing cancellation to abandon unfinished children.
    pub(in crate::execution::vm::tasks) fn poll_owned_groups(
        &mut self,
        mut groups: Vec<u64>,
        mut outcome: Result<Value, VmError>,
    ) -> Result<bool, VmError> {
        let scheduler = Arc::clone(self.scheduler_ref()?);
        while let Some(&group) = groups.first() {
            let Some(failures) = scheduler.poll_group_close(group)? else {
                self.task_suspension = Some(TaskSuspension::OwnerCleanup { groups, outcome });
                return Ok(false);
            };
            for failure in failures {
                if matches!(failure.kind, super::registry::GroupFailureKind::Cancelled) {
                    continue;
                }
                if let Some(mut diagnostic) = failure.diagnostic {
                    let cancelled = matches!(&outcome, Err(error) if error.code == fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED);
                    if outcome.is_ok() || cancelled {
                        diagnostic
                            .help
                            .get_or_insert_with(String::new)
                            .push_str("\nThe creating task was closing its remaining task groups.");
                        outcome = Err(diagnostic);
                    } else if let Err(primary) = &mut outcome
                        && self.observed_task_failure.get() != Some(failure.task)
                    {
                        crate::execution::vm::failure_capture::attach_related(
                            primary,
                            std::slice::from_ref(&diagnostic),
                        );
                        primary
                                .help
                                .get_or_insert_with(String::new)
                                .push_str(&format!(
                                    "\nFurther failure during task group cleanup: Task {} (FP{} at {}:{}): {}",
                                    failure.task, failure.code, failure.line, failure.column, diagnostic.message
                                ));
                    }
                }
            }
            groups.remove(0);
        }
        self.pending_completion = Some(outcome);
        self.observed_task_failure.set(None);
        Ok(true)
    }

    /// Close groups on the main task without executing child code on its stack.
    pub(in crate::execution::vm) fn finish_owned_groups_blocking(
        &mut self,
        outcome: Result<Value, VmError>,
    ) -> Result<Value, VmError> {
        if let Some(value) = self.finish_owned_groups(outcome)? {
            return Ok(value);
        }
        let scheduler = Arc::clone(self.scheduler_ref()?);
        loop {
            let signal = WakeSignal::new();
            let _registration = scheduler.subscribe(&signal);
            let Some(TaskSuspension::OwnerCleanup { groups, outcome }) =
                self.task_suspension.take()
            else {
                unreachable!("owner cleanup retains its continuation until all groups close");
            };
            if self.poll_owned_groups(groups, outcome)? {
                self.suspend_requested = false;
                #[expect(
                    clippy::expect_used,
                    reason = "poll_owned_groups stores the terminal outcome before returning true"
                )]
                return self
                    .pending_completion
                    .take()
                    .expect("completed cleanup retains the outcome");
            }
            signal.wait(Duration::from_millis(10));
        }
    }
}
