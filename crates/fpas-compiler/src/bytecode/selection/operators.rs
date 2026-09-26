//! Typed IR operators mapped to register opcodes.

use super::*;

impl Selector<'_> {
    /// Register opcode for a typed binary operation on `left`.
    pub(super) fn binary_opcode(
        &self,
        operation: BinaryOperation,
        left: ValueId,
    ) -> Result<Opcode, CompileError> {
        let direct = match operation {
            BinaryOperation::AddInteger => Some(Opcode::AddInteger),
            BinaryOperation::SubtractInteger => Some(Opcode::SubtractInteger),
            BinaryOperation::MultiplyInteger => Some(Opcode::MultiplyInteger),
            BinaryOperation::DivideInteger => Some(Opcode::DivideInteger),
            BinaryOperation::RemainderInteger => Some(Opcode::RemainderInteger),
            BinaryOperation::AddReal => Some(Opcode::AddReal),
            BinaryOperation::SubtractReal => Some(Opcode::SubtractReal),
            BinaryOperation::MultiplyReal => Some(Opcode::MultiplyReal),
            BinaryOperation::DivideReal => Some(Opcode::DivideReal),
            BinaryOperation::AddDynamic => Some(Opcode::AddDynamic),
            BinaryOperation::SubtractDynamic => Some(Opcode::SubtractDynamic),
            BinaryOperation::MultiplyDynamic => Some(Opcode::MultiplyDynamic),
            BinaryOperation::DivideDynamic => Some(Opcode::DivideDynamic),
            BinaryOperation::LessThanInteger => Some(Opcode::LessInteger),
            BinaryOperation::GreaterThanInteger => Some(Opcode::GreaterInteger),
            BinaryOperation::LessEqualInteger => Some(Opcode::LessEqualInteger),
            BinaryOperation::GreaterEqualInteger => Some(Opcode::GreaterEqualInteger),
            BinaryOperation::LessThanReal => Some(Opcode::LessReal),
            BinaryOperation::GreaterThanReal => Some(Opcode::GreaterReal),
            BinaryOperation::LessEqualReal => Some(Opcode::LessEqualReal),
            BinaryOperation::GreaterEqualReal => Some(Opcode::GreaterEqualReal),
            BinaryOperation::LessThanString => Some(Opcode::LessString),
            BinaryOperation::GreaterThanString => Some(Opcode::GreaterString),
            BinaryOperation::LessEqualString => Some(Opcode::LessEqualString),
            BinaryOperation::GreaterEqualString => Some(Opcode::GreaterEqualString),
            BinaryOperation::LessThanDynamic => Some(Opcode::LessDynamic),
            BinaryOperation::GreaterThanDynamic => Some(Opcode::GreaterDynamic),
            BinaryOperation::LessEqualDynamic => Some(Opcode::LessEqualDynamic),
            BinaryOperation::GreaterEqualDynamic => Some(Opcode::GreaterEqualDynamic),
            BinaryOperation::AndBoolean => Some(Opcode::AndBoolean),
            BinaryOperation::OrBoolean => Some(Opcode::OrBoolean),
            BinaryOperation::ConcatString => Some(Opcode::ConcatString),
            BinaryOperation::ShiftLeftInteger => Some(Opcode::ShiftLeftInteger),
            BinaryOperation::ShiftRightInteger => Some(Opcode::ShiftRightInteger),
            BinaryOperation::BitAndInteger => Some(Opcode::BitAndInteger),
            BinaryOperation::BitOrInteger => Some(Opcode::BitOrInteger),
            BinaryOperation::BitXorInteger => Some(Opcode::BitXorInteger),
            BinaryOperation::Equal | BinaryOperation::NotEqual => None,
        };
        if let Some(opcode) = direct {
            return Ok(opcode);
        }
        let ty = self
            .value_types
            .get(&left)
            .and_then(|ty| self.program.ty(*ty))
            .map(|definition| &definition.kind)
            .ok_or_else(|| selection_error("equality operand type is missing"))?;
        let equal = operation == BinaryOperation::Equal;
        match (ty, equal) {
            (IrType::Integer, true) => Ok(Opcode::EqualInteger),
            (IrType::Integer, false) => Ok(Opcode::NotEqualInteger),
            (IrType::Real, true) => Ok(Opcode::EqualReal),
            (IrType::Real, false) => Ok(Opcode::NotEqualReal),
            (IrType::Boolean, true) => Ok(Opcode::EqualBoolean),
            (IrType::Boolean, false) => Ok(Opcode::NotEqualBoolean),
            (IrType::String, true) => Ok(Opcode::EqualString),
            (IrType::String, false) => Ok(Opcode::NotEqualString),
            (IrType::Dynamic, true) => Ok(Opcode::EqualDynamic),
            (IrType::Dynamic, false) => Ok(Opcode::NotEqualDynamic),
            (_, true) => Ok(Opcode::EqualDynamic),
            (_, false) => Ok(Opcode::NotEqualDynamic),
        }
    }
}

/// Register opcode for a typed unary operation.
pub(super) fn unary_opcode(operation: UnaryOperation) -> Opcode {
    match operation {
        UnaryOperation::NegateInteger => Opcode::NegateInteger,
        UnaryOperation::NegateReal => Opcode::NegateReal,
        UnaryOperation::NegateDynamic => Opcode::NegateDynamic,
        UnaryOperation::NotBoolean => Opcode::NotBoolean,
        UnaryOperation::IntegerToReal => Opcode::IntegerToReal,
    }
}
