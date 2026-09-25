//! Payload validation for multiword VM superinstructions.

use crate::{FunctionId, FunctionInfo, InstructionAddress, Opcode};

use super::super::{ValidationError, ValidationErrorKind};

pub(super) fn validate_superinstruction_payload(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
) -> Result<(), ValidationError> {
    let expected = match opcode {
        Opcode::BranchIfEqualInteger
        | Opcode::BranchIfNotEqualInteger
        | Opcode::BranchIfLessInteger
        | Opcode::BranchIfGreaterInteger
        | Opcode::BranchIfLessEqualInteger
        | Opcode::BranchIfGreaterEqualInteger => {
            let head = executable.code[address.get() as usize].abc_payload();
            let payload = executable.code.get(address.get() as usize + 1);
            if payload.is_some_and(|word| {
                matches!(
                    word.opcode(),
                    Ok(Opcode::BranchIfTrue | Opcode::BranchIfFalse)
                ) && word.abx_payload().a == head.a
            }) && address.get() + 1 < function.code.end.get()
            {
                return Ok(());
            }
            "matching conditional branch"
        }
        Opcode::ForLoop => {
            let start = address.get() as usize + 1;
            if start + 1 < function.code.end.get() as usize
                && executable.code[start..=start + 1]
                    .iter()
                    .all(|word| word.opcode() == Ok(Opcode::Jump))
            {
                return Ok(());
            }
            "two jump targets"
        }
        _ => return Ok(()),
    };
    Err(ValidationError::instruction(
        executable,
        function_id,
        address,
        Some(opcode),
        ValidationErrorKind::SuperinstructionPayload { expected },
    ))
}
