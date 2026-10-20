//! Equality, ordering, and membership operations.

use std::cmp::Ordering;

use fpas_bytecode::Value;

use super::{BinaryOperation, ValueOperationError};

pub(super) fn binary(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    if let (Value::Integer(left), Value::Integer(right)) = (left, right)
        && operation != BinaryOperation::In
    {
        return super::integer::binary(operation, *left, *right);
    }
    let result = match operation {
        BinaryOperation::Equal => equality(left, right)?,
        BinaryOperation::NotEqual => !equality(left, right)?,
        BinaryOperation::Less => ordering(left, right)?.is_lt(),
        BinaryOperation::LessEqual => ordering(left, right)?.is_le(),
        BinaryOperation::Greater => ordering(left, right)?.is_gt(),
        BinaryOperation::GreaterEqual => ordering(left, right)?.is_ge(),
        BinaryOperation::In => membership(left, right)?,
        _ => unreachable!("scalar operation routed to comparison value operations"),
    };
    Ok(Value::Boolean(result))
}

fn equality(left: &Value, right: &Value) -> Result<bool, ValueOperationError> {
    match (left, right) {
        (Value::Real(left), Value::Real(right)) => {
            fpas_numeric::real::finite(*left).map_err(ValueOperationError::from)?;
            fpas_numeric::real::finite(*right).map_err(ValueOperationError::from)?;
            Ok(left == right)
        }
        (Value::Integer(_), Value::Real(_)) | (Value::Real(_), Value::Integer(_)) => {
            Err(numeric_mixture())
        }
        _ => Ok(left == right),
    }
}

fn ordering(left: &Value, right: &Value) -> Result<Ordering, ValueOperationError> {
    if let Value::Real(value) = left {
        fpas_numeric::real::finite(*value).map_err(ValueOperationError::from)?;
    }
    if let Value::Real(value) = right {
        fpas_numeric::real::finite(*value).map_err(ValueOperationError::from)?;
    }
    if matches!(
        (left, right),
        (Value::Integer(_), Value::Real(_)) | (Value::Real(_), Value::Integer(_))
    ) {
        return Err(numeric_mixture());
    }
    left.scalar_ordering(right).ok_or_else(|| {
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

fn numeric_mixture() -> ValueOperationError {
    ValueOperationError::type_mismatch(
        "Numeric operands must have the same type",
        "Import `uses Std.Conv as Conv;` and use `Conv.IntToReal` explicitly before comparing integer and real values.",
    )
}

fn membership(needle: &Value, aggregate: &Value) -> Result<bool, ValueOperationError> {
    match aggregate {
        Value::Array(values) => Ok(match needle {
            Value::Integer(needle) => values
                .iter()
                .any(|value| matches!(value, Value::Integer(value) if value == needle)),
            _ => values.iter().any(|value| value == needle),
        }),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn array(values: Vec<Value>) -> Value {
        Value::Array(values.into())
    }

    #[test]
    fn integer_membership_preserves_mixed_and_nested_value_semantics() {
        let other_types = array(vec![
            Value::Real(7.0),
            Value::Str("7".into()),
            Value::Boolean(true),
            array(vec![Value::Integer(7)]),
        ]);
        assert!(!membership(&Value::Integer(7), &other_types).expect("membership"));
        assert!(
            membership(&array(vec![Value::Integer(7)]), &other_types).expect("nested membership")
        );
        let integers = array(vec![Value::Integer(i64::MIN), Value::Integer(i64::MAX)]);
        assert!(membership(&Value::Integer(i64::MIN), &integers).expect("minimum"));
        assert!(membership(&Value::Integer(i64::MAX), &integers).expect("maximum"));
        assert!(!membership(&Value::Integer(7), &integers).expect("missing"));
        assert!(!membership(&Value::Integer(0), &array(Vec::new())).expect("empty membership"));
    }
}
