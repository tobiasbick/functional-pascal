//! Scalar arithmetic and Boolean operations.
//!
//! Operator semantics: `docs/pascal/language/basics/operators.md`.

use crate::{SharedStr, Value};

use super::{BinaryOperation, UnaryOperation, ValueOperationError};

/// Applies numeric or Boolean negation with runtime type checks.
pub(super) fn unary(
    operation: UnaryOperation,
    value: &Value,
) -> Result<Value, ValueOperationError> {
    match (operation, value) {
        (UnaryOperation::Negate, Value::Integer(value)) => super::integer::unary(operation, *value),
        (UnaryOperation::Negate, Value::Real(value)) => fpas_numeric::real::finite(-value)
            .map(Value::Real)
            .map_err(Into::into),
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
            if matches!(actual, Value::Integer(_)) {
                "Use `Bits.BitNot(Value)` and import `uses Std.Math.Bits as Bits;` for integer bits."
            } else {
                "Apply `not` only to a Boolean value."
            },
        )),
    }
}

/// Applies arithmetic or Boolean operators with runtime type checks.
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
            boolean(operation, left, right)
        }
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
        (Value::Integer(left), Value::Integer(right)) => {
            super::integer::binary(operation, *left, *right)
        }
        (Value::Real(left), Value::Real(right)) => real_binary(operation, *left, *right),
        _ => Err(numeric_type_error(left, right)),
    }
}

fn real_divide(left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => real_binary(
            BinaryOperation::RealDivide,
            fpas_numeric::real::from_integer(*left),
            fpas_numeric::real::from_integer(*right),
        ),
        (Value::Real(left), Value::Real(right)) => {
            real_binary(BinaryOperation::RealDivide, *left, *right)
        }
        _ => Err(numeric_type_error(left, right)),
    }
}

fn integer_divide(left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(integer_type_error("`div`", left, right));
    };
    super::integer::binary(BinaryOperation::IntegerDivide, *left, *right)
}

fn modulo(left: &Value, right: &Value) -> Result<Value, ValueOperationError> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(integer_type_error("`mod`", left, right));
    };
    super::integer::binary(BinaryOperation::Modulo, *left, *right)
}

fn boolean(
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
        _ => Err(ValueOperationError::type_mismatch(
            format!(
                "Operator requires two Boolean operands, got {} and {}",
                left.type_name(),
                right.type_name()
            ),
            if matches!((left, right), (Value::Integer(_), Value::Integer(_))) {
                let function = match operation {
                    BinaryOperation::And => "BitAnd",
                    BinaryOperation::Or => "BitOr",
                    BinaryOperation::Xor => "BitXor",
                    _ => unreachable!("Boolean operation checked by caller"),
                };
                format!(
                    "Use `Bits.{function}(Left, Right)` and import `uses Std.Math.Bits as Bits;` for integer bits."
                )
            } else {
                "Use two Boolean operands; compare numeric values explicitly, such as `Count > 0`."
                    .to_string()
            },
        )),
    }
}

/// Applies typed real arithmetic with the same bounds as dynamic execution.
pub(crate) fn real_binary(
    operation: BinaryOperation,
    left: f64,
    right: f64,
) -> Result<Value, ValueOperationError> {
    use fpas_numeric::real::{self, Operation};
    let operation = match operation {
        BinaryOperation::Add => Operation::Add,
        BinaryOperation::Subtract => Operation::Subtract,
        BinaryOperation::Multiply => Operation::Multiply,
        BinaryOperation::RealDivide => Operation::Divide,
        _ => unreachable!("real arithmetic operation checked by caller"),
    };
    real::binary(operation, left, right)
        .map(Value::Real)
        .map_err(Into::into)
}

fn numeric_type_error(left: &Value, right: &Value) -> ValueOperationError {
    ValueOperationError::type_mismatch(
        format!(
            "Numeric operation requires two integers or two reals, got {} and {}",
            left.type_name(),
            right.type_name()
        ),
        "Use the same numeric type; import `uses Std.Math as Math;` and use `Math.IntToReal` explicitly to combine an integer with a real.",
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
