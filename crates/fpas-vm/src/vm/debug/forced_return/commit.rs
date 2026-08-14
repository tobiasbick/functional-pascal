//! Prepared multi-frame Worker transition that does not dispatch instructions.

use fpas_bytecode::Value;

use super::super::types::DebugSessionError;
use super::eligibility::{PreparedSelection, prepare_selection};
use super::unsupported;
use crate::vm::worker::Worker;

/// Apply a preflighted selected-frame unwind after a matching structural recheck.
///
/// The live call stack and register window are unchanged when the recheck fails.
pub(in crate::vm::debug) fn commit(
    worker: &mut Worker,
    prepared: &PreparedSelection,
    value: Value,
) -> Result<(), DebugSessionError> {
    let current = prepare_selection(worker, prepared.depth)?;
    if &current != prepared {
        return Err(unsupported(
            "forced return cannot commit because the selected unwind no longer matches stopped state",
            "Request stack frames again for the current stop and retry the selected frame.",
        ));
    }
    apply(worker, prepared, value);
    Ok(())
}

fn apply(worker: &mut Worker, prepared: &PreparedSelection, value: Value) {
    worker.call_stack.truncate(prepared.new_call_stack_len);
    worker.release_registers(prepared.selected_base);
    worker.function = prepared.caller.function;
    worker.ip = prepared.caller.ip;
    worker.base = prepared.caller.base;
    if let Some(destination) = prepared.caller.return_destination {
        worker.registers[destination] = value;
        worker.register_initialized[destination] = true;
    }
}
