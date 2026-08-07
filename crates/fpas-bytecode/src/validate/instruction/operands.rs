//! Reusable contextual operand checks.

use crate::{FunctionId, FunctionInfo, InstructionAddress, NO_REGISTER, Opcode};

use super::super::{ValidationError, ValidationErrorKind};

pub(in crate::validate) fn validate_register(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    operand: &'static str,
    register: u16,
) -> Result<(), ValidationError> {
    if register != NO_REGISTER && register < function.register_count {
        Ok(())
    } else {
        Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            Some(opcode),
            ValidationErrorKind::Register {
                operand,
                actual: register,
                register_count: function.register_count,
            },
        ))
    }
}

pub(super) fn validate_optional_register(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    operand: &'static str,
    register: u16,
) -> Result<(), ValidationError> {
    if register == NO_REGISTER {
        Ok(())
    } else {
        validate_register(
            executable,
            function_id,
            function,
            address,
            opcode,
            operand,
            register,
        )
    }
}

pub(super) fn validate_registers(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    registers: &[(&'static str, u16)],
) -> Result<(), ValidationError> {
    for (operand, register) in registers {
        validate_register(
            executable,
            function_id,
            function,
            address,
            opcode,
            operand,
            *register,
        )?;
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "verifier context keeps diagnostics actionable"
)]
pub(super) fn validate_table_u32(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    table: &'static str,
    operand: &'static str,
    actual: u32,
    length: usize,
) -> Result<(), ValidationError> {
    if usize::try_from(actual)
        .ok()
        .is_some_and(|index| index < length)
    {
        Ok(())
    } else {
        Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            Some(opcode),
            ValidationErrorKind::TableReference {
                table,
                operand,
                actual: u64::from(actual),
                length,
            },
        ))
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "verifier context keeps diagnostics actionable"
)]
pub(super) fn table_u16_error(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    table: &'static str,
    operand: &'static str,
    actual: u16,
    length: usize,
) -> ValidationError {
    ValidationError::instruction(
        executable,
        function_id,
        address,
        Some(opcode),
        ValidationErrorKind::TableReference {
            table,
            operand,
            actual: u64::from(actual),
            length,
        },
    )
}

pub(super) fn canonical_u16(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    operand: &'static str,
    actual: u16,
    expected: u16,
) -> Result<(), ValidationError> {
    canonical(
        executable,
        function_id,
        address,
        opcode,
        operand,
        u64::from(actual),
        u64::from(expected),
    )
}

pub(super) fn canonical_u8(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    operand: &'static str,
    actual: u8,
    expected: u8,
) -> Result<(), ValidationError> {
    canonical(
        executable,
        function_id,
        address,
        opcode,
        operand,
        u64::from(actual),
        u64::from(expected),
    )
}

fn canonical(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    operand: &'static str,
    actual: u64,
    expected: u64,
) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            Some(opcode),
            ValidationErrorKind::NonCanonicalOperand {
                operand,
                actual,
                expected,
            },
        ))
    }
}
