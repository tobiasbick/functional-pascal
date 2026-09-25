//! Fuse typed integer comparisons with their conditional branch.

use fpas_bytecode::{Instruction, Opcode};

use super::super::compile_error;
use crate::CompileError;

pub(crate) fn fuse_integer_branch(
    code: &mut [Instruction],
    branch_address: usize,
) -> Result<(), CompileError> {
    let Some(branch) = code.get(branch_address).copied() else {
        return Ok(());
    };
    if !matches!(
        branch.opcode(),
        Ok(Opcode::BranchIfTrue | Opcode::BranchIfFalse)
    ) {
        return Ok(());
    }
    let previous = code[branch_address - 1];
    let fused = match previous.opcode() {
        Ok(Opcode::EqualInteger) => Opcode::BranchIfEqualInteger,
        Ok(Opcode::NotEqualInteger) => Opcode::BranchIfNotEqualInteger,
        Ok(Opcode::LessInteger) => Opcode::BranchIfLessInteger,
        Ok(Opcode::GreaterInteger) => Opcode::BranchIfGreaterInteger,
        Ok(Opcode::LessEqualInteger) => Opcode::BranchIfLessEqualInteger,
        Ok(Opcode::GreaterEqualInteger) => Opcode::BranchIfGreaterEqualInteger,
        _ => return Ok(()),
    };
    let operands = previous.abc_payload();
    if operands.a != branch.abx_payload().a {
        return Ok(());
    }
    code[branch_address - 1] = Instruction::abc(
        fused,
        operands.a,
        operands.b,
        operands.c,
        operands.auxiliary,
    )
    .map_err(|error| compile_error(&error.to_string()))?;
    Ok(())
}
