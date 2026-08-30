//! Integer arithmetic, bitwise, shift, and comparison primitives.

use fpas_bytecode::Value;

use super::{BinaryOperation, UnaryOperation, ValueOperationError};

#[inline]
pub(super) fn unary(operation: UnaryOperation, value: i64) -> Result<Value, ValueOperationError> {
    match operation {
        UnaryOperation::Negate => value.checked_neg().map(Value::Integer).ok_or_else(|| {
            ValueOperationError::domain(
                "Integer negation overflow",
                "Avoid negating the minimum integer value.",
            )
        }),
        UnaryOperation::Not => unreachable!("Boolean operation routed to integer value operations"),
    }
}

#[inline]
pub(super) fn binary(
    operation: BinaryOperation,
    left: i64,
    right: i64,
) -> Result<Value, ValueOperationError> {
    match operation {
        BinaryOperation::Add => Ok(Value::Integer(left.wrapping_add(right))),
        BinaryOperation::Subtract => Ok(Value::Integer(left.wrapping_sub(right))),
        BinaryOperation::Multiply => Ok(Value::Integer(left.wrapping_mul(right))),
        BinaryOperation::IntegerDivide => integer_divide(left, right),
        BinaryOperation::Modulo => modulo(left, right),
        BinaryOperation::And => Ok(Value::Integer(left & right)),
        BinaryOperation::Or => Ok(Value::Integer(left | right)),
        BinaryOperation::Xor => Ok(Value::Integer(left ^ right)),
        BinaryOperation::ShiftLeft | BinaryOperation::ShiftRight => shift(operation, left, right),
        BinaryOperation::Equal => Ok(Value::Boolean(left == right)),
        BinaryOperation::NotEqual => Ok(Value::Boolean(left != right)),
        BinaryOperation::Less => Ok(Value::Boolean(left < right)),
        BinaryOperation::LessEqual => Ok(Value::Boolean(left <= right)),
        BinaryOperation::Greater => Ok(Value::Boolean(left > right)),
        BinaryOperation::GreaterEqual => Ok(Value::Boolean(left >= right)),
        BinaryOperation::RealDivide | BinaryOperation::In => {
            unreachable!("non-integer operation routed to integer value operations")
        }
    }
}

fn integer_divide(left: i64, right: i64) -> Result<Value, ValueOperationError> {
    if right == 0 {
        return Err(ValueOperationError::division_by_zero(
            "Division by zero",
            "Check the right-hand side before using `div` or `/`.",
        ));
    }
    left.checked_div(right).map(Value::Integer).ok_or_else(|| {
        ValueOperationError::domain(
            "Integer division overflow",
            "Avoid dividing the minimum integer value by `-1`.",
        )
    })
}

fn modulo(left: i64, right: i64) -> Result<Value, ValueOperationError> {
    if right == 0 {
        return Err(ValueOperationError::modulo_by_zero(
            "Modulo by zero",
            "Check the right-hand side before using `mod`.",
        ));
    }
    left.checked_rem(right).map(Value::Integer).ok_or_else(|| {
        ValueOperationError::domain(
            "Integer modulo overflow",
            "Avoid applying `mod` with the minimum integer value and `-1`.",
        )
    })
}

fn shift(
    operation: BinaryOperation,
    value: i64,
    amount: i64,
) -> Result<Value, ValueOperationError> {
    let amount = u32::try_from(amount)
        .ok()
        .filter(|amount| *amount < 64)
        .ok_or_else(|| {
            ValueOperationError::domain(
                format!("Shift amount {amount} is out of range (0..63)"),
                "Use a shift amount between 0 and 63 inclusive.",
            )
        })?;
    Ok(Value::Integer(match operation {
        BinaryOperation::ShiftLeft => value.wrapping_shl(amount),
        BinaryOperation::ShiftRight => value.wrapping_shr(amount),
        _ => unreachable!("shift operation checked by caller"),
    }))
}
