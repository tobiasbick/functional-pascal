//! Calls in tail position that reuse the caller's frame.

use fpas_bytecode::{Instruction, NO_REGISTER, Opcode};
use fpas_ir::{Function, IrType, Program, Terminator, TypeId};

use super::super::allocation::Allocation;
use super::super::compile_error;
use crate::CompileError;

/// Rewrite a block-ending call into a tail call when `terminator` returns its result.
///
/// `words` are the selected words of the block's last instruction `last`. A direct call becomes
/// `TailCall` when caller and callee share their return convention; a unit call drops the trailing
/// `LoadUnit`, because `Return` without a value never reads it. A call through a function value
/// becomes `TailCallValue`; its target is known only at run time, so the VM checks the convention
/// there. The `Return` word stays: the verifier requires it, and debugger-owned execution runs it.
pub(crate) fn convert_tail_call(
    program: &Program,
    caller: &Function,
    allocation: &Allocation,
    last: &fpas_ir::Instruction,
    words: &mut Vec<Instruction>,
    terminator: &Terminator,
) -> Result<(), CompileError> {
    let Terminator::Return(returned) = terminator else {
        return Ok(());
    };
    let returned_register = match returned {
        Some(value) => allocation.value(*value)?.get(),
        None => NO_REGISTER,
    };
    match words.last().map(|word| word.opcode()) {
        Some(Ok(Opcode::CallValue)) => {
            let operands = words[words.len() - 1].abc_payload();
            let destination = match returned {
                Some(_) if operands.a == returned_register => operands.a,
                // The unit result register is never read after the call.
                None if last
                    .result
                    .is_some_and(|result| is_unit(program, result.ty)) =>
                {
                    NO_REGISTER
                }
                _ => return Ok(()),
            };
            replace_last(words, Opcode::TailCallValue, destination)
        }
        Some(Ok(Opcode::CallDirect)) if returned.is_some() => {
            convert_direct(program, caller, words, words.len() - 1, returned_register)
        }
        Some(Ok(Opcode::LoadUnit)) if returned.is_none() && words.len() >= 2 => {
            convert_direct(program, caller, words, words.len() - 2, returned_register)
        }
        _ => Ok(()),
    }
}

fn convert_direct(
    program: &Program,
    caller: &Function,
    words: &mut Vec<Instruction>,
    call_index: usize,
    returned_register: u16,
) -> Result<(), CompileError> {
    let call = words[call_index];
    if call.opcode() != Ok(Opcode::CallDirect) {
        return Ok(());
    }
    let operands = call.abc_payload();
    let Some(callee) = program.functions.get(usize::from(operands.b)) else {
        return Ok(());
    };
    if operands.a != returned_register
        || is_unit(program, callee.signature.result) != is_unit(program, caller.signature.result)
    {
        return Ok(());
    }
    words.truncate(call_index + 1);
    replace_last(words, Opcode::TailCall, operands.a)
}

fn replace_last(
    words: &mut [Instruction],
    opcode: Opcode,
    destination: u16,
) -> Result<(), CompileError> {
    let Some(last) = words.last_mut() else {
        return Ok(());
    };
    let operands = last.abc_payload();
    *last = Instruction::abc(
        opcode,
        destination,
        operands.b,
        operands.c,
        operands.auxiliary,
    )
    .map_err(|error| compile_error(&error.to_string()))?;
    Ok(())
}

fn is_unit(program: &Program, ty: TypeId) -> bool {
    matches!(
        program.ty(ty).map(|definition| &definition.kind),
        Some(IrType::Unit)
    )
}
