//! Prepared multi-frame Worker transition that does not dispatch instructions.

use fpas_bytecode::{InstructionAddress, Opcode, Value};

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
    apply_prepared(worker, prepared, value);
    Ok(())
}

/// Apply an already revalidated unwind without another fallible step.
pub(in crate::vm::debug) fn apply_prepared(
    worker: &mut Worker,
    prepared: &PreparedSelection,
    value: Value,
) {
    worker.call_stack.truncate(prepared.new_call_stack_len);
    worker.release_registers(prepared.selected_base);
    worker.function = prepared.caller.function;
    worker.ip = prepared.caller.ip;
    worker.base = prepared.caller.base;
    if let Some(destination) = prepared.caller.return_destination {
        worker.registers[destination] = value;
        worker.register_initialized[destination] = true;
        complete_declared_initializer_move(worker, destination);
    }
}

fn complete_declared_initializer_move(worker: &mut Worker, source: usize) {
    let Ok(address) = InstructionAddress::try_from_index(worker.ip) else {
        return;
    };
    let image = worker.executable.executable();
    let Some(function) = image.functions.get(usize::from(worker.function.get())) else {
        return;
    };
    let Some(binding) = function
        .debug
        .bindings
        .iter()
        .find(|binding| binding.initializer == Some(address))
    else {
        return;
    };
    let Some(instruction) = image.code.get(worker.ip).copied() else {
        return;
    };
    if instruction.opcode().ok() != Some(Opcode::Move) {
        return;
    }
    let operands = instruction.abc_payload();
    if worker.base.saturating_add(usize::from(operands.b)) != source
        || operands.a != binding.register.get()
    {
        return;
    }
    worker.ip = worker.ip.saturating_add(1);
    if worker.take_suppressed_source_initializer(address) {
        return;
    }
    let destination = worker.base.saturating_add(usize::from(operands.a));
    worker.registers[destination] = worker.registers[source].clone();
    worker.register_initialized[destination] = true;
}
