//! Opcode-form decoding and top-level instruction validation.

mod abc;
mod operands;

use crate::validate::site::InstructionSite;
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
    let site = InstructionSite {
        executable,
        function_id,
        function,
        address,
        opcode,
    };
    match opcode.form() {
        InstructionForm::Abc => {
            let operands = instruction
                .abc_operands()
                .map_err(|error| instruction_error(site, error))?;
            validate_abc(site, operands)?;
        }
        InstructionForm::Abx => {
            let operands = instruction
                .abx_operands()
                .map_err(|error| instruction_error(site, error))?;
            validate_abx(site, operands)?;
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
    site: InstructionSite<'_>,
    operands: crate::AbxOperands,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable, opcode, ..
    } = site;
    let crate::AbxOperands { a, bx } = operands;
    match opcode {
        Opcode::LoadConstant => {
            validate_register(site, "destination", a)?;
            validate_table_u32(
                site,
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
            validate_register(site, operand, a)?;
            validate_table_u32(site, "globals", "global", bx, executable.globals.len())
        }
        Opcode::Jump => canonical_u16(site, "A", a, 0),
        Opcode::BranchIfFalse | Opcode::BranchIfTrue => validate_register(site, "condition", a),
        _ => Err(instruction_error(
            site,
            InstructionError::FormMismatch {
                opcode,
                expected: opcode.form(),
                actual: InstructionForm::Abx,
            },
        )),
    }
}

fn instruction_error(site: InstructionSite<'_>, error: InstructionError) -> ValidationError {
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
        ValidationErrorKind::Instruction(error),
    )
}
