//! Centralized opcode operand relocation.

use fpas_bytecode::Op;
use fpas_unit::object::RelocationKind;

pub(super) fn relocate_instruction(
    op: &mut Op,
    relocation: RelocationKind,
    constants: &[u16],
    code_base: u32,
) -> Result<(), ()> {
    match relocation {
        RelocationKind::Constant { operand, index } => {
            let mapped = *constants.get(index as usize).ok_or(())?;
            relocate_constant(op, operand, mapped)
        }
        RelocationKind::CodeAddress { target } => {
            let mapped = code_base.checked_add(target).ok_or(())?;
            match op {
                Op::Jump(value)
                | Op::JumpIfFalse(value)
                | Op::JumpIfTrue(value)
                | Op::JumpIfLocalGt(_, _, value)
                | Op::JumpIfLocalLt(_, _, value) => {
                    *value = mapped;
                    Ok(())
                }
                _ => Err(()),
            }
        }
    }
}

fn relocate_constant(op: &mut Op, operand: u8, mapped: u16) -> Result<(), ()> {
    match (op, operand) {
        (
            Op::Constant(index)
            | Op::GetGlobal(index)
            | Op::SetGlobal(index)
            | Op::GlobalIndexSet(index, _)
            | Op::Call(index, _)
            | Op::MakeClosure(index, _)
            | Op::MakeRecord(index, _)
            | Op::FieldGet(index)
            | Op::FieldSet(index),
            0,
        ) => {
            *index = mapped;
            Ok(())
        }
        (Op::MakeEnum(index, _, _) | Op::IsVariant(index, _), 0) => {
            *index = mapped;
            Ok(())
        }
        (Op::MakeEnum(_, index, _) | Op::IsVariant(_, index), 1) => {
            *index = mapped;
            Ok(())
        }
        _ => Err(()),
    }
}
