//! Prepared register-window reconstruction for selected frame restart.

use fpas_bytecode::{FunctionId, InstructionAddress, Value};

use super::unsupported;
use crate::vm::debug::types::DebugSessionError;
use crate::vm::worker::Worker;

/// Fully validated restart state whose application cannot fail.
pub(in crate::vm::debug) struct PreparedFrameRestart {
    function: FunctionId,
    start: InstructionAddress,
    start_index: usize,
    base: usize,
    register_end: usize,
    new_call_stack_len: usize,
    prefix: Vec<Value>,
    discarded_frames: usize,
}

impl PreparedFrameRestart {
    /// Number of younger frames removed by the prepared restart.
    pub(in crate::vm::debug) const fn discarded_frames(&self) -> usize {
        self.discarded_frames
    }
}

/// Validate and capture the current argument/capture prefix for one selected frame.
pub(in crate::vm::debug) fn prepare(
    worker: &Worker,
    depth: usize,
) -> Result<PreparedFrameRestart, DebugSessionError> {
    let saved_len = worker.call_stack.len();
    let new_call_stack_len = saved_len.checked_sub(depth).ok_or_else(|| {
        unsupported(
            "frame restart selected a depth outside the current call stack",
            "Request stack frames again for the current stop and select one current frame.",
        )
    })?;
    let (function, base) = if depth == 0 {
        (worker.function, worker.base)
    } else {
        let selected = worker.call_stack.get(new_call_stack_len).ok_or_else(|| {
            unsupported(
                "frame restart cannot identify the selected saved frame",
                "Request stack frames again for the current stop and retry.",
            )
        })?;
        (selected.function, selected.base)
    };
    let info = worker
        .executable
        .executable()
        .functions
        .get(usize::from(function.get()))
        .ok_or_else(|| {
            unsupported(
                "frame restart cannot find the selected function metadata",
                "Rebuild the executable with the current compiler and retry.",
            )
        })?;
    let prefix_count = usize::from(info.arity).saturating_add(usize::from(info.capture_count));
    let register_count = usize::from(info.register_count);
    if prefix_count > register_count {
        return Err(unsupported(
            "frame restart found an invalid argument and capture prefix",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    let register_end = base.checked_add(register_count).ok_or_else(|| {
        unsupported(
            "frame restart register bounds overflow this host",
            "Rebuild the executable with the current compiler and retry.",
        )
    })?;
    let prefix_end = base.checked_add(prefix_count).ok_or_else(|| {
        unsupported(
            "frame restart prefix bounds overflow this host",
            "Rebuild the executable with the current compiler and retry.",
        )
    })?;
    if register_end > worker.active_register_count
        || register_end > worker.registers.len()
        || register_end > worker.register_initialized.len()
    {
        return Err(unsupported(
            "frame restart selected an incomplete live register window",
            "Request a current live frame or rebuild the executable with matching metadata.",
        ));
    }
    if worker.register_initialized[base..prefix_end]
        .iter()
        .any(|initialized| !initialized)
    {
        return Err(unsupported(
            "frame restart requires every current parameter and capture to be initialized",
            "Select a live frame whose arguments and captures are available.",
        ));
    }
    let start_index = usize::try_from(info.code.start.get()).map_err(|_| {
        unsupported(
            "frame restart function entry does not fit this host",
            "Rebuild the executable for this target and retry.",
        )
    })?;
    Ok(PreparedFrameRestart {
        function,
        start: info.code.start,
        start_index,
        base,
        register_end,
        new_call_stack_len,
        prefix: worker.registers[base..prefix_end].to_vec(),
        discarded_frames: depth,
    })
}

/// Apply one fully validated restart without executing bytecode or returning an error.
pub(in crate::vm::debug) fn apply(worker: &mut Worker, prepared: PreparedFrameRestart) {
    worker.call_stack.truncate(prepared.new_call_stack_len);
    worker.release_registers(prepared.base);
    worker.activate_registers(prepared.register_end);
    for (offset, value) in prepared.prefix.into_iter().enumerate() {
        let register = prepared.base + offset;
        worker.registers[register] = value;
        worker.register_initialized[register] = true;
    }
    worker.function = prepared.function;
    worker.ip = prepared.start_index;
    worker.base = prepared.base;
    worker.current_address = prepared.start;
    worker.task_suspension = None;
    worker.suspend_requested = false;
}
