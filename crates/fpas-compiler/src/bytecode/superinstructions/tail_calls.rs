//! Direct calls in tail position that reuse the caller's frame.

use fpas_bytecode::{Instruction, NO_REGISTER, Opcode};
use fpas_ir::{Function, IrType, Program, Terminator};

use super::super::allocation::Allocation;
use super::super::compile_error;
use crate::CompileError;

/// Rewrite a block-ending `CallDirect` into `TailCall` when `terminator` returns its result.
///
/// `words` are the selected words of the block's last instruction. A unit call drops the
/// trailing `LoadUnit`, because `Return` without a value never reads it. The `Return` word stays;
/// the verifier requires it after `TailCall`, and debugger-owned execution runs it.
pub(crate) fn convert_tail_call(
    program: &Program,
    caller: &Function,
    allocation: &Allocation,
    words: &mut Vec<Instruction>,
    terminator: &Terminator,
) -> Result<(), CompileError> {
    let Terminator::Return(returned) = terminator else {
        return Ok(());
    };
    let call_index = match (returned, words.last().map(|word| word.opcode())) {
        (Some(_), Some(Ok(Opcode::CallDirect))) => words.len() - 1,
        (None, Some(Ok(Opcode::LoadUnit))) if words.len() >= 2 => words.len() - 2,
        _ => return Ok(()),
    };
    let call = words[call_index];
    if call.opcode() != Ok(Opcode::CallDirect) {
        return Ok(());
    }
    let operands = call.abc_payload();
    let returned_register = match returned {
        Some(value) => allocation.value(*value)?.get(),
        None => NO_REGISTER,
    };
    let Some(callee) = program.functions.get(usize::from(operands.b)) else {
        return Ok(());
    };
    if operands.a != returned_register
        || returns_unit(program, callee) != returns_unit(program, caller)
    {
        return Ok(());
    }
    words.truncate(call_index);
    words.push(
        Instruction::abc(
            Opcode::TailCall,
            operands.a,
            operands.b,
            operands.c,
            operands.auxiliary,
        )
        .map_err(|error| compile_error(&error.to_string()))?,
    );
    Ok(())
}

fn returns_unit(program: &Program, function: &Function) -> bool {
    matches!(
        program
            .ty(function.signature.result)
            .map(|definition| &definition.kind),
        Some(IrType::Unit)
    )
}
