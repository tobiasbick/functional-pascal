//! Contain group-owned failures while preserving ordinary debugger failure stops.

use super::{DebugTaskEvent, DebugTaskEventKind, DebugTaskState, TaskScheduler, TaskSlot};
use crate::vm::VmError;

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
    if owned && !scheduler.is_aborted() {
        // A published group outcome is terminal and cannot be resumed by debugger recovery.
        slot.worker.task_suspension = None;
        slot.exited = true;
        events.push(DebugTaskEvent {
            task_id,
            kind: DebugTaskEventKind::Exited,
        });
        true
    } else {
        false
    }
}
