//! Prepared completion of a program or task entry frame.

use fpas_bytecode::FunctionId;

use super::unsupported;
use crate::vm::debug::types::DebugSessionError;
use crate::vm::worker::Worker;

/// Stable entry-frame identity retained across result evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::vm::debug) struct PreparedEntryCompletion {
    /// Task that owns the selected entry.
    pub task_id: u64,
    /// Entry function selected from the bottom logical frame.
    pub function: FunctionId,
    /// Entry register base.
    pub base: usize,
    /// Exact saved-call-stack length at preparation time.
    pub call_stack_len: usize,
}

/// Prove that `depth` selects the entry frame and capture its commit identity.
pub(in crate::vm::debug) fn prepare_entry(
    worker: &Worker,
    task_id: u64,
    depth: usize,
) -> Result<PreparedEntryCompletion, DebugSessionError> {
    if depth != worker.call_stack.len() {
        return Err(unsupported(
            "forced entry completion requires the program or task entry frame",
            "Request the current stack and select its oldest frame.",
        ));
    }
    let (function, base) = worker
        .call_stack
        .first()
        .map_or((worker.function, worker.base), |frame| {
            (frame.function, frame.base)
        });
    if base > worker.active_register_count {
        return Err(unsupported(
            "forced entry completion cannot release an invalid register window",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    Ok(PreparedEntryCompletion {
        task_id,
        function,
        base,
        call_stack_len: worker.call_stack.len(),
    })
}
