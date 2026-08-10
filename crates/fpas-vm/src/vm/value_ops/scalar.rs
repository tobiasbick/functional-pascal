//! Scalar arithmetic, Boolean, bitwise, and shift operations.

use fpas_bytecode::{SharedStr, Value};

use super::{BinaryOperation, UnaryOperation, ValueOperationError};

pub(super) fn unary(
    operation: UnaryOperation,
    value: &Value,
) -> Result<Value, ValueOperationError> {
    match (operation, value) {
        (UnaryOperation::Negate, Value::Integer(value)) => {
            value.checked_neg().map(Value::Integer).ok_or_else(|| {
                ValueOperationError::domain(
                    "Integer negation overflow",
                    "Avoid negating the minimum integer value.",
                )
            })
        }
        (UnaryOperation::Negate, Value::Real(value)) => Ok(Value::Real(-value)),
        (UnaryOperation::Not, Value::Boolean(value)) => Ok(Value::Boolean(!value)),
        (UnaryOperation::Negate, actual) => Err(ValueOperationError::type_mismatch(
            format!(
                "Cannot negate non-numeric value of type {}",
                actual.type_name()
            ),
            "Apply unary `-` only to integer or real values.",
        )),
        (UnaryOperation::Not, actual) => Err(ValueOperationError::type_mismatch(
            format!("Cannot apply `not` to value of type {}", actual.type_name()),
            "Apply `not` only to a Boolean value.",
        )),
    }
}

pub(super) fn binary(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    match operation {
        BinaryOperation::Add | BinaryOperation::Subtract | BinaryOperation::Multiply => {
            arithmetic(operation, left, right)
        }
        BinaryOperation::RealDivide => real_divide(left, right),
        BinaryOperation::IntegerDivide => integer_divide(left, right),
        BinaryOperation::Modulo => modulo(left, right),
        BinaryOperation::And | BinaryOperation::Or | BinaryOperation::Xor => {
            boolean_or_bitwise(operation, left, right)
        }
        BinaryOperation::ShiftLeft | BinaryOperation::ShiftRight => shift(operation, left, right),
        _ => unreachable!("comparison operation routed to scalar value operations"),
    }
}

fn arithmetic(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    if operation == BinaryOperation::Add
        && let (Value::Str(left), Value::Str(right)) = (left, right)
    {
        return Ok(Value::Str(SharedStr::concat(left, right)));
    }
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(match operation {
            BinaryOperation::Add => left.wrapping_add(*right),
            BinaryOperation::Subtract => left.wrapping_sub(*right),
            BinaryOperation::Multiply => left.wrapping_mul(*right),
            _ => unreachable!("arithmetic operation checked by caller"),
        })),
        _ => {
            let (Some(left), Some(right)) = (numeric(left), numeric(right)) else {
                return Err(numeric_type_error(left, right));
            };
            Ok(Value::Real(match operation {
                BinaryOperation::Add => left + right,
                BinaryOperation::Subtract => left - right,
                BinaryOperation::Multiply => left * right,
                _ => unreachable!("arithmetic operation checked by caller"),
            }))
        }
    }
}

fn real_divide(left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    let (Some(left), Some(right)) = (numeric(left), numeric(right)) else {
        return Err(numeric_type_error(left, right));
    };
    if right == 0.0 {
        return Err(ValueOperationError::division_by_zero(
            "Division by zero",
            "Check the right-hand side before using `/`.",
        ));
    }
    Ok(Value::Real(left / right))
}

fn integer_divide(left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(integer_type_error("`div`", left, right));
    };
    if *right == 0 {
        return Err(ValueOperationError::division_by_zero(
            "Division by zero",
            "Check the right-hand side before using `div` or `/`.",
        ));
    }
    left.checked_div(*right).map(Value::Integer).ok_or_else(|| {
        ValueOperationError::domain(
            "Integer division overflow",
            "Avoid dividing the minimum integer value by `-1`.",
        )
    })
}

fn modulo(left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(integer_type_error("`mod`", left, right));
    };
    if *right == 0 {
        return Err(ValueOperationError::modulo_by_zero(
            "Modulo by zero",
            "Check the right-hand side before using `mod`.",
        ));
    }
    left.checked_rem(*right).map(Value::Integer).ok_or_else(|| {
        ValueOperationError::domain(
            "Integer modulo overflow",
            "Avoid applying `mod` with the minimum integer value and `-1`.",
        )
    })
}

fn boolean_or_bitwise(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    match (left, right) {
        (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(match operation {
            BinaryOperation::And => *left && *right,
            BinaryOperation::Or => *left || *right,
            BinaryOperation::Xor => *left ^ *right,
            _ => unreachable!("Boolean operation checked by caller"),
        })),
        (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(match operation {
            BinaryOperation::And => *left & *right,
            BinaryOperation::Or => *left | *right,
            BinaryOperation::Xor => *left ^ *right,
            _ => unreachable!("bitwise operation checked by caller"),
        })),
        _ => Err(ValueOperationError::type_mismatch(
            format!(
                "Operator requires two Boolean or two integer operands, got {} and {}",
                left.type_name(),
                right.type_name()
            ),
            "Use matching Boolean or integer operands.",
        )),
    }
}

fn shift(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    let (Value::Integer(value), Value::Integer(amount)) = (left, right) else {
        return Err(integer_type_error("shift", left, right));
    };
    let amount = u32::try_from(*amount)
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

fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Real(value) => Some(*value),
        _ => None,
    }
}

fn numeric_type_error(left: &Value, right: &Value) -> ValueOperationError {
    ValueOperationError::type_mismatch(
        format!(
            "Numeric operation requires numeric operands, got {} and {}",
            left.type_name(),
            right.type_name()
        ),
        "Ensure both operands are integer or real values.",
    )
}

fn integer_type_error(operation: &str, left: &Value, right: &Value) -> ValueOperationError {
    ValueOperationError::type_mismatch(
        format!(
            "{operation} requires integer operands, got {} and {}",
            left.type_name(),
            right.type_name()
        ),
        "Use integer operands for this operation.",
    )
}
