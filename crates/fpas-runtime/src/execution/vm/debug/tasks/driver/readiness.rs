//! Readiness transitions and explicit progress for controlled quiescence.

use super::*;
use crate::execution::vm::tasks::TaskSuspension;
use std::sync::atomic::Ordering;

impl DebugTaskRuntime {
    #[inline]
    pub(super) fn refresh_readiness(&mut self) -> Result<bool, (u64, VmError)> {
        let mut progress = false;
        for (&task_id, slot) in &mut self.tasks {
            if slot.worker.trace_interrupted() {
                return Ok(progress);
            }
            if !matches!(
                slot.state,
                DebugTaskState::Waiting | DebugTaskState::Sleeping | DebugTaskState::Runnable
            ) || slot.worker.task_suspension.is_none()
            {
                continue;
            }
            progress |= poll_suspended(task_id, slot, &self.clock, &mut self.resumed_at_boundary)?;
            if slot.worker.trace_interrupted() {
                return Ok(progress);
            }
        }
        Ok(progress)
    }
}

// Keep full polling out of the scheduler's common runnable-task traversal.
#[inline(never)]
fn poll_suspended(
    task_id: u64,
    slot: &mut TaskSlot,
    clock: &TaskClock,
    resumed_at_boundary: &mut BTreeSet<u64>,
) -> Result<bool, (u64, VmError)> {
    // Nothing can commit until input arrives or the deadline expires. Cancellation
    // still wins at the same boundary; failures use the guarded poll/unwind path.
    if unchanged_wait(&slot.worker, clock) && slot.worker.check_task_cancellation().is_ok() {
        return Ok(false);
    }
    let access = slot
        .worker
        .execution_access()
        .map_err(|error| (task_id, error))?;
    let before = readiness_progress(slot);
    let mut progress = false;
    match slot.worker.poll_task_suspension() {
        Ok(true) => {
            progress = true;
            slot.state = DebugTaskState::Runnable;
            if slot.worker.pending_completion.is_none() {
                resumed_at_boundary.insert(task_id);
            }
        }
        Ok(false) => slot.state = state_from_suspension(&slot.worker),
        Err(error) => {
            progress = true;
            let error = match slot.worker.begin_cleanup_failure(error) {
                Ok(()) => {
                    slot.state = DebugTaskState::Runnable;
                    return Ok(true);
                }
                Err(error) => error,
            };
            if slot.worker.begin_scope_unwind(error.clone()).is_ok() {
                slot.state = state_from_suspension(&slot.worker);
                if (access.outermost() || slot.worker.domain.maintenance_pending())
                    && slot.worker.publish_execution(&[]).is_ok()
                {
                    access.safe_point();
                }
                return Ok(true);
            }
            // Retain the ordinary dispatch boundary for terminal scope cleanup.
            slot.worker.pending_completion = Some(Err(error));
            slot.state = DebugTaskState::Runnable;
        }
    }
    if (access.outermost() || slot.worker.domain.maintenance_pending())
        && slot.worker.publish_execution(&[]).is_ok()
    {
        access.safe_point();
    }
    Ok(progress || before != readiness_progress(slot))
}

fn unchanged_wait(worker: &Worker, clock: &TaskClock) -> bool {
    match &worker.task_suspension {
        Some(TaskSuspension::ConsoleRead {
            deadline_millis,
            input_revision,
            observed_revision,
            ..
        }) => {
            clock.now_millis() < *deadline_millis
                && input_revision.load(Ordering::Acquire) == *observed_revision
        }
        Some(TaskSuspension::Sleep { deadline_millis }) => clock.now_millis() < *deadline_millis,
        _ => false,
    }
}

// Every successful source commitment consumes its suspension. Partial owner cleanup
// and scope unwinding can change shared state while retaining a blocked suspension.
fn readiness_progress(
    slot: &TaskSlot,
) -> (
    DebugTaskState,
    usize,
    usize,
    Option<std::mem::Discriminant<crate::execution::vm::tasks::TaskSuspension>>,
) {
    let groups = match &slot.worker.task_suspension {
        Some(crate::execution::vm::tasks::TaskSuspension::OwnerCleanup { groups, .. }) => {
            groups.len()
        }
        _ => 0,
    };
    (
        slot.state,
        slot.worker.task_scopes.len(),
        groups,
        slot.worker
            .task_suspension
            .as_ref()
            .map(std::mem::discriminant),
    )
}
