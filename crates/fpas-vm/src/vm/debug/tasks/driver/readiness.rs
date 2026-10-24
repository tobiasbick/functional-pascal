//! Readiness transitions and explicit progress for controlled quiescence.

use super::*;

impl DebugTaskRuntime {
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
            let access = slot
                .worker
                .execution_access()
                .map_err(|error| (task_id, error))?;
            let before = readiness_progress(slot);
            match slot.worker.poll_task_suspension() {
                Ok(true) => {
                    progress = true;
                    slot.state = DebugTaskState::Runnable;
                    if slot.worker.pending_completion.is_none() {
                        self.resumed_at_boundary.insert(task_id);
                    }
                }
                Ok(false) => slot.state = state_from_suspension(&slot.worker),
                Err(error) => {
                    progress = true;
                    if slot.worker.begin_scope_unwind(error.clone()).is_ok() {
                        slot.state = state_from_suspension(&slot.worker);
                        if (access.outermost() || slot.worker.domain.maintenance_pending())
                            && slot.worker.publish_execution(&[]).is_ok()
                        {
                            access.safe_point();
                        }
                        continue;
                    }
                    // Complete group cleanup and supervision through the same
                    // terminal boundary as ordinary instruction dispatch.
                    slot.worker.pending_completion = Some(Err(error));
                    slot.state = DebugTaskState::Runnable;
                }
            }
            if (access.outermost() || slot.worker.domain.maintenance_pending())
                && slot.worker.publish_execution(&[]).is_ok()
            {
                access.safe_point();
            }
            progress |= before != readiness_progress(slot);
            if slot.worker.trace_interrupted() {
                return Ok(progress);
            }
        }
        Ok(progress)
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
    Option<std::mem::Discriminant<crate::vm::tasks::TaskSuspension>>,
) {
    let groups = match &slot.worker.task_suspension {
        Some(crate::vm::tasks::TaskSuspension::OwnerCleanup { groups, .. }) => groups.len(),
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
