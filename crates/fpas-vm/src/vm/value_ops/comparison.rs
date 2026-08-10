//! Equality, ordering, and membership operations.

use std::cmp::Ordering;

use fpas_bytecode::Value;

use super::{BinaryOperation, ValueOperationError};

pub(super) fn binary(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    let result = match operation {
        BinaryOperation::Equal => equality(left, right),
        BinaryOperation::NotEqual => !equality(left, right),
        BinaryOperation::Less => ordering(left, right)?.is_lt(),
        BinaryOperation::LessEqual => ordering(left, right)?.is_le(),
        BinaryOperation::Greater => ordering(left, right)?.is_gt(),
        BinaryOperation::GreaterEqual => ordering(left, right)?.is_ge(),
        BinaryOperation::In => membership(left, right)?,
        _ => unreachable!("scalar operation routed to comparison value operations"),
    };
    Ok(Value::Boolean(result))
}

fn equality(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Real(left), Value::Real(right)) => left == right,
        (Value::Integer(left), Value::Real(right)) => (*left as f64) == *right,
        (Value::Real(left), Value::Integer(right)) => *left == (*right as f64),
        _ => left == right,
    }
}

fn ordering(left: &Value, right: &Value) -> Result<Ordering, ValueOperationError> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Some(left.cmp(right)),
        (Value::Boolean(left), Value::Boolean(right)) => Some(left.cmp(right)),
        (Value::Str(left), Value::Str(right)) => Some(left.cmp(right)),
        (Value::Integer(left), Value::Real(right)) => (*left as f64).partial_cmp(right),
        (Value::Real(left), Value::Integer(right)) => left.partial_cmp(&(*right as f64)),
        (Value::Real(left), Value::Real(right)) => left.partial_cmp(right),
        _ => None,
    }
    .ok_or_else(|| {
        ValueOperationError::type_mismatch(
            format!(
                "Ordered comparison requires compatible scalar operands, got {} and {}",
                left.type_name(),
                right.type_name()
            ),
            "Compare two numeric, Boolean, or string values.",
        )
    })
}

fn membership(needle: &Value, aggregate: &Value) -> Result<bool, ValueOperationError> {
    match aggregate {
        Value::Array(values) => Ok(values.iter().any(|value| value == needle)),
        Value::Dict(pairs) => Ok(pairs.iter().any(|(key, _)| key == needle)),
        Value::Str(text) => {
            let Value::Str(value) = needle else {
                return Err(ValueOperationError::type_mismatch(
                    "String membership requires a string value",
                    "Use `Substring in Text` for string membership.",
                ));
            };
            let mut characters = value.chars();
            Ok(match (characters.next(), characters.next()) {
                (Some(character), None) => text.chars().any(|candidate| candidate == character),
                _ => text.contains(value.as_ref()),
            })
        }
        other => Err(ValueOperationError::type_mismatch(
            format!(
                "Membership requires an array, dictionary, or string, got {}",
                other.type_name()
            ),
            "Use `in` with an array, dictionary, or string on the right-hand side.",
        )),
    }
}
