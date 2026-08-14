//! Stop, task, and selected-frame eligibility plus prepared unwind proof.

use fpas_bytecode::{FunctionId, InstructionAddress};

use super::super::inspection::DebugFrame;
use super::super::types::{DebugSessionError, DebugSessionState, DebugStopReason};
use super::unsupported;
use crate::vm::debug::types::DebugTaskState;
use crate::vm::frame::CallFrame;
use crate::vm::worker::Worker;

/// Verified selected-frame unwind retained for the commit-time structural recheck.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::vm::debug) struct PreparedSelection {
    /// Inspection depth of the selected frame.
    pub depth: usize,
    /// Function that owns the selected frame.
    pub selected_function: FunctionId,
    /// Register base of the selected frame.
    pub selected_base: usize,
    /// Saved caller restored after the unwind.
    pub caller: CallFrame,
    /// Selected frame plus every younger discarded frame.
    pub unwind_count: usize,
    /// Call-stack length after truncating the discarded suffix.
    pub new_call_stack_len: usize,
}

/// Reject stops, frames, and tasks outside the accepted forced-return slice.
pub(in crate::vm::debug) fn require_eligible(
    state: DebugSessionState,
    stop_reason: DebugStopReason,
    stop_task_id: u64,
    frame: &DebugFrame,
    frame_task_id: u64,
    task_state: Option<DebugTaskState>,
    worker: &Worker,
) -> Result<(), DebugSessionError> {
    if state == DebugSessionState::Failed || stop_reason == DebugStopReason::RuntimeError {
        return Err(unsupported(
            "forced return is not available after a runtime-error stop",
            "Clear the failure by restarting the debug session; this command is not exception recovery.",
        ));
    }
    if frame_task_id != stop_task_id {
        return Err(unsupported(
            format!(
                "forced return is not available for task {frame_task_id}; the current stop belongs to task {stop_task_id}"
            ),
            "Select a non-entry frame of the task that caused the current stop.",
        ));
    }
    match task_state {
        Some(DebugTaskState::Waiting | DebugTaskState::Sleeping) => {
            return Err(unsupported(
                "forced return is not available for a waiting or sleeping task",
                "Wait until the selected task is the runnable task that caused the current stop.",
            ));
        }
        Some(DebugTaskState::Failed | DebugTaskState::Cancelled | DebugTaskState::Completed) => {
            return Err(unsupported(
                "forced return is not available for a failed, cancelled, or completed task",
                "Select a non-entry frame of the task that caused the current non-failure stop.",
            ));
        }
        Some(DebugTaskState::Runnable | DebugTaskState::Running) | None => {}
    }
    if frame.depth >= worker.call_stack.len() {
        return Err(entry_unsupported());
    }
    Ok(())
}

/// Map a selected inspection depth onto a fully checked unwind.
pub(in crate::vm::debug) fn prepare_selection(
    worker: &Worker,
    depth: usize,
) -> Result<PreparedSelection, DebugSessionError> {
    let saved_len = worker.call_stack.len();
    let unwind_count = depth.checked_add(1).ok_or_else(entry_unsupported)?;
    let new_call_stack_len = saved_len
        .checked_sub(unwind_count)
        .ok_or_else(entry_unsupported)?;
    let (selected_function, selected_base) = selected_identity(worker, depth, saved_len)?;
    let caller = worker
        .call_stack
        .get(new_call_stack_len)
        .copied()
        .ok_or_else(entry_unsupported)?;
    prove_restore(worker, selected_base, &caller)?;
    Ok(PreparedSelection {
        depth,
        selected_function,
        selected_base,
        caller,
        unwind_count,
        new_call_stack_len,
    })
}

fn selected_identity(
    worker: &Worker,
    depth: usize,
    saved_len: usize,
) -> Result<(FunctionId, usize), DebugSessionError> {
    if depth == 0 {
        return Ok((worker.function, worker.base));
    }
    let selected_index = saved_len.checked_sub(depth).ok_or_else(entry_unsupported)?;
    let selected = worker
        .call_stack
        .get(selected_index)
        .ok_or_else(entry_unsupported)?;
    Ok((selected.function, selected.base))
}

fn prove_restore(
    worker: &Worker,
    selected_base: usize,
    caller: &CallFrame,
) -> Result<(), DebugSessionError> {
    let image = worker.executable.executable();
    let Some(caller_info) = image.functions.get(usize::from(caller.function.get())) else {
        return Err(unsupported(
            "forced return cannot restore a caller with missing function metadata",
            "Rebuild the executable with the current compiler and retry.",
        ));
    };
    if !ip_in_function(caller.ip, caller_info.code.start, caller_info.code.end) {
        return Err(unsupported(
            "forced return cannot restore a caller instruction pointer outside its function",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    if caller.base > selected_base || selected_base > worker.active_register_count {
        return Err(unsupported(
            "forced return cannot release selected and younger register windows because release bounds are invalid",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    if let Some(destination) = caller.return_destination
        && (destination < caller.base
            || destination >= selected_base
            || destination >= worker.active_register_count
            || destination >= worker.registers.len()
            || destination >= worker.register_initialized.len())
    {
        return Err(unsupported(
            "forced return cannot write the result because the saved destination is outside the selected frame's caller window",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    Ok(())
}

fn entry_unsupported() -> DebugSessionError {
    unsupported(
        "forced return is not available for a program or task entry frame",
        "Select a non-entry frame that has a saved caller before using forced return.",
    )
}

fn ip_in_function(ip: usize, start: InstructionAddress, end: InstructionAddress) -> bool {
    let Ok(start) = usize::try_from(start.get()) else {
        return false;
    };
    let Ok(end) = usize::try_from(end.get()) else {
        return false;
    };
    ip >= start && ip < end
}
