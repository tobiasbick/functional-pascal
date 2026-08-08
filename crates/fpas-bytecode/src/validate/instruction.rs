//! Opcode-form decoding and top-level instruction validation.

mod abc;
mod operands;

use crate::{
    FunctionId, FunctionInfo, InstructionAddress, InstructionError, InstructionForm, Opcode,
};

use self::abc::validate_abc;
pub(super) use self::operands::validate_register;
use self::operands::{canonical_u16, validate_table_u32};
use super::{ValidationError, ValidationErrorKind};

pub(super) fn validate_instruction(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
) -> Result<Opcode, ValidationError> {
    let Some(instruction) = usize::try_from(address.get())
        .ok()
        .and_then(|index| executable.code.get(index))
        .copied()
    else {
        return Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            None,
            ValidationErrorKind::CodeRange {
                start: function.code.start.get(),
                end: function.code.end.get(),
                code: executable.code.len(),
            },
        ));
    };
    let opcode = instruction.opcode().map_err(|error| {
        ValidationError::instruction(
            executable,
            function_id,
            address,
            None,
            ValidationErrorKind::Instruction(error),
        )
    })?;
    match opcode.form() {
        InstructionForm::Abc => {
            let operands = instruction.abc_operands().map_err(|error| {
                instruction_error(executable, function_id, address, opcode, error)
            })?;
            validate_abc(executable, function_id, function, address, opcode, operands)?;
        }
        InstructionForm::Abx => {
            let operands = instruction.abx_operands().map_err(|error| {
                instruction_error(executable, function_id, address, opcode, error)
            })?;
            validate_abx(executable, function_id, function, address, opcode, operands)?;
        }
        InstructionForm::Ax => {
            return Err(ValidationError::instruction(
                executable,
                function_id,
                address,
                Some(opcode),
                ValidationErrorKind::ReservedOpcode,
            ));
        }
    }
    Ok(opcode)
}

fn validate_abx(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    operands: crate::AbxOperands,
) -> Result<(), ValidationError> {
    let crate::AbxOperands { a, bx } = operands;
    match opcode {
        Opcode::LoadConstant => {
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            validate_table_u32(
                executable,
                function_id,
                address,
                opcode,
                "constants",
                "constant",
                bx,
                executable.constants.len(),
            )
        }
        Opcode::LoadGlobal | Opcode::StoreGlobal => {
            let operand = if opcode == Opcode::LoadGlobal {
                "destination"
            } else {
                "value"
            };
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                operand,
                a,
            )?;
            validate_table_u32(
                executable,
                function_id,
                address,
                opcode,
                "globals",
                "global",
                bx,
                executable.globals.len(),
            )
        }
        Opcode::Jump => canonical_u16(executable, function_id, address, opcode, "A", a, 0),
        Opcode::BranchIfFalse | Opcode::BranchIfTrue => validate_register(
            executable,
            function_id,
            function,
            address,
            opcode,
            "condition",
            a,
        ),
        _ => Err(instruction_error(
            executable,
            function_id,
            address,
            opcode,
            InstructionError::FormMismatch {
                opcode,
                expected: opcode.form(),
                actual: InstructionForm::Abx,
            },
        )),
    }
}

fn instruction_error(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    error: InstructionError,
) -> ValidationError {
    ValidationError::instruction(
        executable,
        function_id,
        address,
        Some(opcode),
        ValidationErrorKind::Instruction(error),
    )
}
