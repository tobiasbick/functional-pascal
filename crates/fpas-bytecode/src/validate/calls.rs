//! Call targets, arity, destinations, and contiguous argument windows.

use crate::validate::site::InstructionSite;
use crate::{NO_REGISTER, ReturnConvention};

use super::{ValidationError, ValidationErrorKind};

pub(super) struct CallOperands {
    pub destination: u16,
    pub target: u16,
    pub argument_base: u16,
    pub argument_count: u8,
}

pub(super) fn validate_call(
    site: InstructionSite<'_>,
    operands: CallOperands,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id: caller_id,
        address,
        opcode,
        ..
    } = site;
    let Some(target) = executable.functions.get(usize::from(operands.target)) else {
        return Err(ValidationError::instruction(
            executable,
            caller_id,
            address,
            Some(opcode),
            ValidationErrorKind::TableReference {
                table: "functions",
                operand: "function",
                actual: u64::from(operands.target),
                length: executable.functions.len(),
            },
        ));
    };
    if target.arity != operands.argument_count {
        return Err(ValidationError::instruction(
            executable,
            caller_id,
            address,
            Some(opcode),
            ValidationErrorKind::CallArity {
                target: operands.target,
                expected: target.arity,
                actual: operands.argument_count,
            },
        ));
    }
    validate_destination(site, operands.destination, target.return_convention)?;
    validate_window(
        site,
        "argument window",
        operands.argument_base,
        usize::from(operands.argument_count),
    )
}

pub(super) fn validate_destination(
    site: InstructionSite<'_>,
    destination: u16,
    convention: ReturnConvention,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id,
        function,
        address,
        opcode,
    } = site;
    match convention {
        ReturnConvention::Unit if destination == NO_REGISTER => Ok(()),
        ReturnConvention::Value if destination < function.register_count => Ok(()),
        _ => Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            Some(opcode),
            ValidationErrorKind::ReturnConvention {
                expected: convention,
                actual: destination,
            },
        )),
    }
}

pub(super) fn validate_window(
    site: InstructionSite<'_>,
    operand: &'static str,
    base: u16,
    count: usize,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id,
        function,
        address,
        opcode,
    } = site;
    let end = usize::from(base).checked_add(count);
    if base != NO_REGISTER && end.is_some_and(|end| end <= usize::from(function.register_count)) {
        return Ok(());
    }
    Err(ValidationError::instruction(
        executable,
        function_id,
        address,
        Some(opcode),
        ValidationErrorKind::RegisterWindow {
            operand,
            base,
            count,
            register_count: function.register_count,
        },
    ))
}
