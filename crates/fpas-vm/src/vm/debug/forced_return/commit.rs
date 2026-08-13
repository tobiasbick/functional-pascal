//! Preflighted one-frame Worker transition that does not dispatch instructions.

use fpas_bytecode::{InstructionAddress, Value};

use super::super::types::DebugSessionError;
use super::unsupported;
use crate::vm::frame::CallFrame;
use crate::vm::worker::Worker;

/// Pop exactly one saved caller after structural preflight.
///
/// The live call stack and register window are unchanged when preflight fails.
pub(in crate::vm::debug) fn commit(
    worker: &mut Worker,
    value: Value,
) -> Result<(), DebugSessionError> {
    let frame = preflight(worker)?;
    let callee_base = worker.base;
    let _ = worker.call_stack.pop();
    worker.release_registers(callee_base);
    worker.function = frame.function;
    worker.ip = frame.ip;
    worker.base = frame.base;
    if let Some(destination) = frame.return_destination {
        worker.registers[destination] = value;
        worker.register_initialized[destination] = true;
    }
    Ok(())
}

fn preflight(worker: &Worker) -> Result<CallFrame, DebugSessionError> {
    let Some(frame) = worker.call_stack.last().copied() else {
        return Err(unsupported(
            "forced return is not available for a program or task entry frame",
            "Step into an ordinary callee that has a saved caller before using forced return.",
        ));
    };
    let image = worker.executable.executable();
    let Some(caller) = image.functions.get(usize::from(frame.function.get())) else {
        return Err(unsupported(
            "forced return cannot restore a caller with missing function metadata",
            "Rebuild the executable with the current compiler and retry.",
        ));
    };
    if !ip_in_function(frame.ip, caller.code.start, caller.code.end) {
        return Err(unsupported(
            "forced return cannot restore a caller instruction pointer outside its function",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    if let Some(destination) = frame.return_destination
        && (destination < frame.base
            || destination >= worker.base
            || destination >= worker.active_register_count
            || destination >= worker.registers.len()
            || destination >= worker.register_initialized.len())
    {
        return Err(unsupported(
            "forced return cannot write the result because the saved destination is outside the caller window",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    if worker.base > worker.active_register_count {
        return Err(unsupported(
            "forced return cannot release the callee register window because release bounds are invalid",
            "Rebuild the executable with the current compiler and retry.",
        ));
    }
    Ok(frame)
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
