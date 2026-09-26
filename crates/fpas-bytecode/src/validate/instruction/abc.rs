//! ABC-form semantic operand checks.

use crate::validate::site::InstructionSite;
use crate::{InstructionError, InstructionForm, Opcode};

use super::super::calls::{CallOperands, validate_call, validate_destination, validate_window};
use super::super::layouts::validate_layout_operand;
use super::super::{ValidationError, ValidationErrorKind};
use super::operands::{
    canonical_u8, canonical_u16, table_u16_error, validate_optional_register, validate_register,
    validate_registers,
};

/// Validates the semantic operands of one decoded ABC instruction.
pub(super) fn validate_abc(
    site: InstructionSite<'_>,
    operands: crate::AbcOperands,
) -> Result<(), ValidationError> {
    let InstructionSite {
        executable,
        function_id,
        function,
        address,
        opcode,
    } = site;
    let crate::AbcOperands { a, b, c, auxiliary } = operands;
    if validate_layout_operand(site, a, b, c, auxiliary)? {
        return Ok(());
    }
    match opcode {
        Opcode::AddInteger
        | Opcode::SubtractInteger
        | Opcode::MultiplyInteger
        | Opcode::DivideInteger
        | Opcode::RemainderInteger
        | Opcode::AddReal
        | Opcode::SubtractReal
        | Opcode::MultiplyReal
        | Opcode::DivideReal
        | Opcode::AddDynamic
        | Opcode::SubtractDynamic
        | Opcode::MultiplyDynamic
        | Opcode::DivideDynamic
        | Opcode::EqualDynamic
        | Opcode::NotEqualDynamic
        | Opcode::LessDynamic
        | Opcode::GreaterDynamic
        | Opcode::LessEqualDynamic
        | Opcode::GreaterEqualDynamic
        | Opcode::ShiftLeftInteger
        | Opcode::ShiftRightInteger
        | Opcode::BitAndInteger
        | Opcode::BitOrInteger
        | Opcode::BitXorInteger
        | Opcode::EqualInteger
        | Opcode::NotEqualInteger
        | Opcode::LessInteger
        | Opcode::GreaterInteger
        | Opcode::LessEqualInteger
        | Opcode::GreaterEqualInteger
        | Opcode::EqualReal
        | Opcode::NotEqualReal
        | Opcode::LessReal
        | Opcode::GreaterReal
        | Opcode::LessEqualReal
        | Opcode::GreaterEqualReal
        | Opcode::EqualString
        | Opcode::NotEqualString
        | Opcode::LessString
        | Opcode::GreaterString
        | Opcode::LessEqualString
        | Opcode::GreaterEqualString
        | Opcode::EqualBoolean
        | Opcode::NotEqualBoolean
        | Opcode::AndBoolean
        | Opcode::OrBoolean
        | Opcode::BranchIfEqualInteger
        | Opcode::BranchIfNotEqualInteger
        | Opcode::BranchIfLessInteger
        | Opcode::BranchIfGreaterInteger
        | Opcode::BranchIfLessEqualInteger
        | Opcode::BranchIfGreaterEqualInteger
        | Opcode::IndexGet
        | Opcode::IndexSet
        | Opcode::Contains => {
            validate_registers(site, &[("destination", a), ("left", b), ("right", c)])?;
            canonical_u8(site, "auxiliary", auxiliary, 0)
        }
        Opcode::ConcatString => {
            validate_registers(site, &[("destination", a), ("left", b), ("right", c)])?;
            if auxiliary <= 1 {
                Ok(())
            } else {
                canonical_u8(site, "consume-left flag", auxiliary, 1)
            }
        }
        Opcode::AddIntegerImm | Opcode::DivideIntegerImm => {
            validate_registers(site, &[("destination", a), ("left", b)])?;
            canonical_u8(site, "auxiliary", auxiliary, 0)
        }
        Opcode::ForLoop => {
            validate_registers(site, &[("counter", a), ("bound", b)])?;
            canonical_u16(site, "C", c, 0)?;
            if auxiliary <= 1 {
                Ok(())
            } else {
                canonical_u8(site, "direction", auxiliary, 1)
            }
        }
        Opcode::ArrayPop
        | Opcode::NegateInteger
        | Opcode::NegateReal
        | Opcode::NegateDynamic
        | Opcode::NotBoolean
        | Opcode::IntegerToReal
        | Opcode::MakeCell
        | Opcode::CellRead
        | Opcode::MakeOk
        | Opcode::MakeError
        | Opcode::MakeSome
        | Opcode::IsResultOk
        | Opcode::IsOptionSome
        | Opcode::UnwrapOk
        | Opcode::UnwrapError
        | Opcode::UnwrapSome => {
            validate_registers(site, &[("destination", a), ("source", b)])?;
            canonical_tail(site, c, auxiliary)
        }
        Opcode::Move => {
            validate_registers(site, &[("destination", a), ("source", b)])?;
            canonical_u16(site, "C", c, 0)?;
            if auxiliary <= 1 {
                Ok(())
            } else {
                canonical_u8(site, "consume-source flag", auxiliary, 1)
            }
        }
        Opcode::LoadUnit | Opcode::MakeNone => {
            validate_register(site, "destination", a)?;
            canonical_u16(site, "B", b, 0)?;
            canonical_tail(site, c, auxiliary)
        }
        Opcode::CellWrite => {
            validate_registers(site, &[("cell", a), ("value", b)])?;
            canonical_tail(site, c, auxiliary)
        }
        Opcode::Return => {
            validate_destination(site, a, function.return_convention)?;
            canonical_u16(site, "B", b, 0)?;
            canonical_tail(site, c, auxiliary)
        }
        Opcode::Panic => {
            validate_register(site, "panic value", a)?;
            canonical_u16(site, "B", b, 0)?;
            canonical_tail(site, c, auxiliary)
        }
        Opcode::CallDirect | Opcode::TailCall => validate_call(
            site,
            CallOperands {
                destination: a,
                target: b,
                argument_base: c,
                argument_count: auxiliary,
            },
        ),
        Opcode::CallValue | Opcode::TailCallValue | Opcode::SpawnTask => {
            validate_optional_register(site, "destination", a)?;
            validate_register(site, "callee", b)?;
            validate_window(site, "argument window", c, usize::from(auxiliary))
        }
        Opcode::SpawnDetachedTask => {
            validate_register(site, "callee", a)?;
            validate_window(site, "argument window", b, usize::from(auxiliary))?;
            canonical_u16(site, "C", c, 0)
        }
        Opcode::MakeClosure => validate_closure(site, a, b, c, auxiliary),
        Opcode::MakeArray => {
            validate_register(site, "destination", a)?;
            validate_window(site, "array value window", b, usize::from(c))?;
            canonical_u8(site, "auxiliary", auxiliary, 0)
        }
        Opcode::MakeDictionary => {
            validate_register(site, "destination", a)?;
            let count = usize::from(c)
                .checked_mul(2)
                .ok_or_else(|| window_error(site, b))?;
            validate_window(site, "dictionary pair window", b, count)?;
            canonical_u8(site, "auxiliary", auxiliary, 0)
        }
        Opcode::ArrayPush => {
            validate_registers(site, &[("destination", a), ("array", b), ("value", c)])?;
            canonical_u8(site, "auxiliary", auxiliary, 0)
        }
        Opcode::StoreGlobalIndexPath => {
            validate_register(site, "global snapshot", a)?;
            if executable.globals.get(usize::from(b)).is_none() {
                return Err(table_u16_error(
                    site,
                    "globals",
                    "global",
                    b,
                    executable.globals.len(),
                ));
            }
            validate_window(
                site,
                "global index path window",
                c,
                usize::from(auxiliary).saturating_add(1),
            )
        }
        Opcode::Intrinsic => {
            validate_optional_register(site, "destination", a)?;
            if crate::Intrinsic::from_u16(b).is_none() {
                return Err(ValidationError::instruction(
                    executable,
                    function_id,
                    address,
                    Some(opcode),
                    ValidationErrorKind::UnknownIntrinsic { actual: b },
                ));
            }
            validate_window(site, "intrinsic argument window", c, usize::from(auxiliary))
        }
        Opcode::Yield => {
            canonical_u16(site, "A", a, 0)?;
            canonical_u16(site, "B", b, 0)?;
            canonical_tail(site, c, auxiliary)
        }
        Opcode::LoadConstant
        | Opcode::Jump
        | Opcode::BranchIfFalse
        | Opcode::BranchIfTrue
        | Opcode::LoadGlobal
        | Opcode::StoreGlobal
        | Opcode::MakeRecord
        | Opcode::LoadField
        | Opcode::StoreField
        | Opcode::UpdateRecord
        | Opcode::MakeEnum
        | Opcode::TestVariant
        | Opcode::LoadEnumField => Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            Some(opcode),
            ValidationErrorKind::Instruction(InstructionError::FormMismatch {
                opcode,
                expected: opcode.form(),
                actual: InstructionForm::Abc,
            }),
        )),
    }
}

fn validate_closure(
    site: InstructionSite<'_>,
    destination: u16,
    target: u16,
    capture_base: u16,
    capture_count: u8,
) -> Result<(), ValidationError> {
    let InstructionSite { executable, .. } = site;
    validate_register(site, "destination", destination)?;
    let Some(target_info) = executable.functions.get(usize::from(target)) else {
        return Err(table_u16_error(
            site,
            "functions",
            "function",
            target,
            executable.functions.len(),
        ));
    };
    if usize::from(target_info.capture_count) != usize::from(capture_count) {
        return Err(window_error(site, capture_base));
    }
    validate_window(
        site,
        "capture window",
        capture_base,
        usize::from(capture_count),
    )
}

fn canonical_tail(site: InstructionSite<'_>, c: u16, auxiliary: u8) -> Result<(), ValidationError> {
    canonical_u16(site, "C", c, 0)?;
    canonical_u8(site, "auxiliary", auxiliary, 0)
}

fn window_error(site: InstructionSite<'_>, base: u16) -> ValidationError {
    let InstructionSite {
        executable,
        function_id,
        function,
        address,
        opcode,
    } = site;
    ValidationError::instruction(
        executable,
        function_id,
        address,
        Some(opcode),
        ValidationErrorKind::RegisterWindow {
            operand: "capture or value window",
            base,
            count: usize::MAX,
            register_count: function.register_count,
        },
    )
}
