//! Selection of integer operations with adjacent literal operands.

use fpas_ir::{BinaryOperation, Constant, Instruction, Operation, ValueId};

/// Return an encodable literal from the immediately preceding IR instruction.
pub(crate) fn integer_immediate(
    previous: Option<&Instruction>,
    operation: BinaryOperation,
    left: ValueId,
    right: ValueId,
) -> Option<i16> {
    if !matches!(
        operation,
        BinaryOperation::AddInteger | BinaryOperation::DivideInteger
    ) {
        return None;
    }
    let previous = previous?;
    let (Operation::Const(Constant::Integer(value)), Some(constant)) =
        (&previous.operation, previous.result.map(|result| result.id))
    else {
        return None;
    };
    if right != constant || left == constant {
        return None;
    }
    i16::try_from(*value).ok()
}
