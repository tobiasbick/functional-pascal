//! Publish group and scope outcomes while preserving ordinary debugger failure stops.

use super::{DebugTaskEvent, DebugTaskEventKind, DebugTaskState, TaskScheduler, TaskSlot};
use crate::execution::vm::VmError;
use fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED;

/// Publish a terminal outcome after the task has left all of its open scopes.
pub(super) fn record(
    slot: &mut TaskSlot,
    task_id: u64,
    error: &VmError,
    scheduler: &TaskScheduler,
    events: &mut Vec<DebugTaskEvent>,
) -> bool {
    slot.state = DebugTaskState::Failed;
    slot.worker.supervision = None;
    slot.failure = Some(error.clone());
    let owned = slot.worker.retain_result && scheduler.store_failure(task_id, error.clone());
    let cancelled = task_id != 0 && error.code == RUNTIME_TASK_CANCELLED;
    if (owned || cancelled) && !scheduler.is_aborted() {
        // Published owned outcomes are terminal and cannot be resumed by debugger recovery.
        if error.code == RUNTIME_TASK_CANCELLED {
            slot.state = DebugTaskState::Cancelled;
            slot.failure = None;
        }
        slot.worker.task_suspension = None;
        slot.retire_execution();
        slot.exited = true;
        events.push(DebugTaskEvent {
            task_id,
            kind: DebugTaskEventKind::Exited,
        });
        true
    } else {
        let _ = slot.worker.publish_execution(&[]);
        false
    }
}
