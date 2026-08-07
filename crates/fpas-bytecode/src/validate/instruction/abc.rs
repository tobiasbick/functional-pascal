//! ABC-form semantic operand checks.

use crate::{
    FunctionId, FunctionInfo, InstructionAddress, InstructionError, InstructionForm, Opcode,
};

use super::super::calls::{CallOperands, validate_call, validate_destination, validate_window};
use super::super::layouts::validate_layout_operand;
use super::super::{ValidationError, ValidationErrorKind};
use super::operands::{
    canonical_u8, canonical_u16, table_u16_error, validate_optional_register, validate_register,
    validate_registers,
};

pub(super) fn validate_abc(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    operands: crate::AbcOperands,
) -> Result<(), ValidationError> {
    let crate::AbcOperands { a, b, c, auxiliary } = operands;
    if validate_layout_operand(
        executable,
        function_id,
        function,
        address,
        opcode,
        a,
        b,
        c,
        auxiliary,
    )? {
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
        | Opcode::ConcatString
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
        | Opcode::IndexGet
        | Opcode::IndexSet
        | Opcode::Contains => {
            validate_registers(
                executable,
                function_id,
                function,
                address,
                opcode,
                &[("destination", a), ("left", b), ("right", c)],
            )?;
            canonical_u8(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )
        }
        Opcode::Move
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
            validate_registers(
                executable,
                function_id,
                function,
                address,
                opcode,
                &[("destination", a), ("source", b)],
            )?;
            canonical_tail(executable, function_id, address, opcode, c, auxiliary)
        }
        Opcode::LoadUnit | Opcode::MakeNone => {
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            canonical_u16(executable, function_id, address, opcode, "B", b, 0)?;
            canonical_tail(executable, function_id, address, opcode, c, auxiliary)
        }
        Opcode::CellWrite => {
            validate_registers(
                executable,
                function_id,
                function,
                address,
                opcode,
                &[("cell", a), ("value", b)],
            )?;
            canonical_tail(executable, function_id, address, opcode, c, auxiliary)
        }
        Opcode::Return => {
            validate_destination(
                executable,
                function_id,
                function,
                address,
                opcode,
                a,
                function.return_convention,
            )?;
            canonical_u16(executable, function_id, address, opcode, "B", b, 0)?;
            canonical_tail(executable, function_id, address, opcode, c, auxiliary)
        }
        Opcode::Panic => {
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "panic value",
                a,
            )?;
            canonical_u16(executable, function_id, address, opcode, "B", b, 0)?;
            canonical_tail(executable, function_id, address, opcode, c, auxiliary)
        }
        Opcode::CallDirect => validate_call(
            executable,
            function_id,
            function,
            address,
            opcode,
            CallOperands {
                destination: a,
                target: b,
                argument_base: c,
                argument_count: auxiliary,
            },
        ),
        Opcode::CallValue | Opcode::SpawnTask => {
            validate_optional_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "callee",
                b,
            )?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "argument window",
                c,
                usize::from(auxiliary),
            )
        }
        Opcode::SpawnDetachedTask => {
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "callee",
                a,
            )?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "argument window",
                b,
                usize::from(auxiliary),
            )?;
            canonical_u16(executable, function_id, address, opcode, "C", c, 0)
        }
        Opcode::MakeClosure => validate_closure(
            executable,
            function_id,
            function,
            address,
            opcode,
            a,
            b,
            c,
            auxiliary,
        ),
        Opcode::MakeArray => {
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "array value window",
                b,
                usize::from(c),
            )?;
            canonical_u8(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )
        }
        Opcode::MakeDictionary => {
            validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            let count = usize::from(c).checked_mul(2).ok_or_else(|| {
                window_error(executable, function_id, function, address, opcode, b)
            })?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "dictionary pair window",
                b,
                count,
            )?;
            canonical_u8(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )
        }
        Opcode::Intrinsic => {
            validate_optional_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            if crate::Intrinsic::from_u16(b).is_none() {
                return Err(ValidationError::instruction(
                    executable,
                    function_id,
                    address,
                    Some(opcode),
                    ValidationErrorKind::UnknownIntrinsic { actual: b },
                ));
            }
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "intrinsic argument window",
                c,
                usize::from(auxiliary),
            )
        }
        Opcode::Yield => {
            canonical_u16(executable, function_id, address, opcode, "A", a, 0)?;
            canonical_u16(executable, function_id, address, opcode, "B", b, 0)?;
            canonical_tail(executable, function_id, address, opcode, c, auxiliary)
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
        | Opcode::LoadEnumField
        | Opcode::ReservedMetadata => Err(ValidationError::instruction(
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

#[expect(clippy::too_many_arguments, reason = "closure verifier context")]
fn validate_closure(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    destination: u16,
    target: u16,
    capture_base: u16,
    capture_count: u8,
) -> Result<(), ValidationError> {
    validate_register(
        executable,
        function_id,
        function,
        address,
        opcode,
        "destination",
        destination,
    )?;
    let Some(target_info) = executable.functions.get(usize::from(target)) else {
        return Err(table_u16_error(
            executable,
            function_id,
            address,
            opcode,
            "functions",
            "function",
            target,
            executable.functions.len(),
        ));
    };
    if usize::from(target_info.capture_count) != usize::from(capture_count) {
        return Err(window_error(
            executable,
            function_id,
            function,
            address,
            opcode,
            capture_base,
        ));
    }
    validate_window(
        executable,
        function_id,
        function,
        address,
        opcode,
        "capture window",
        capture_base,
        usize::from(capture_count),
    )
}

fn canonical_tail(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    c: u16,
    auxiliary: u8,
) -> Result<(), ValidationError> {
    canonical_u16(executable, function_id, address, opcode, "C", c, 0)?;
    canonical_u8(
        executable,
        function_id,
        address,
        opcode,
        "auxiliary",
        auxiliary,
        0,
    )
}

fn window_error(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    base: u16,
) -> ValidationError {
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
