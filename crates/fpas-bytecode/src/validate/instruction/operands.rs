//! Reusable contextual operand checks.

use crate::NO_REGISTER;
use crate::validate::site::InstructionSite;

use super::super::{ValidationError, ValidationErrorKind};

pub(in crate::validate) fn validate_register(
    site: InstructionSite<'_>,
    operand: &'static str,
    register: u16,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id,
        function,
        address,
        opcode,
    } = site;
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
    site: InstructionSite<'_>,
    operand: &'static str,
    register: u16,
) -> Result<(), ValidationError> {
    if register == NO_REGISTER {
        Ok(())
    } else {
        validate_register(site, operand, register)
    }
}

pub(super) fn validate_registers(
    site: InstructionSite<'_>,
    registers: &[(&'static str, u16)],
) -> Result<(), ValidationError> {
    for (operand, register) in registers {
        validate_register(site, operand, *register)?;
    }
    Ok(())
}

pub(super) fn validate_table_u32(
    site: InstructionSite<'_>,
    table: &'static str,
    operand: &'static str,
    actual: u32,
    length: usize,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id,
        address,
        opcode,
        ..
    } = site;
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

pub(super) fn table_u16_error(
    site: InstructionSite<'_>,
    table: &'static str,
    operand: &'static str,
    actual: u16,
    length: usize,
) -> ValidationError {
    let InstructionSite {
        executable,
        function_id,
        address,
        opcode,
        ..
    } = site;
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
    site: InstructionSite<'_>,
    operand: &'static str,
    actual: u16,
    expected: u16,
) -> Result<(), ValidationError> {
    canonical(site, operand, u64::from(actual), u64::from(expected))
}

pub(super) fn canonical_u8(
    site: InstructionSite<'_>,
    operand: &'static str,
    actual: u8,
    expected: u8,
) -> Result<(), ValidationError> {
    canonical(site, operand, u64::from(actual), u64::from(expected))
}

fn canonical(
    site: InstructionSite<'_>,
    operand: &'static str,
    actual: u64,
    expected: u64,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id,
        address,
        opcode,
        ..
    } = site;
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
