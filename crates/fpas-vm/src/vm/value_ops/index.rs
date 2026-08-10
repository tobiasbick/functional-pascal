//! Read-only aggregate field and index operations.

use fpas_bytecode::Value;

use super::ValueOperationError;

pub(super) fn field(value: &Value, name: &str) -> Result<Value, ValueOperationError> {
    let (type_name, fields, values) = match value {
        Value::Record(record) => (
            record.body().layout.type_name.as_str(),
            record.body().layout.fields.as_slice(),
            record.body().values.as_slice(),
        ),
        Value::Enum(enumeration) => (
            enumeration.body().layout.type_name.as_str(),
            enumeration.body().layout.fields.as_slice(),
            enumeration.body().values.as_slice(),
        ),
        other => {
            return Err(ValueOperationError::type_mismatch(
                format!("Cannot read field `{name}` from {}", other.type_name()),
                "Read stored fields only from record or enum values.",
            ));
        }
    };
    fields
        .iter()
        .position(|field| field.eq_ignore_ascii_case(name))
        .and_then(|index| values.get(index))
        .cloned()
        .ok_or_else(|| {
            ValueOperationError::domain(
                format!("Stored field `{name}` does not exist on {type_name}"),
                "Use a stored record or enum field visible in the Variables view.",
            )
        })
}

pub(super) fn index(value: &Value, key: &Value) -> Result<Value, ValueOperationError> {
    match value {
        Value::Array(values) => {
            let index = checked_index(key, "array")?;
            values.get(index).cloned().ok_or_else(|| {
                ValueOperationError::array_bounds(
                    format!("Array index {index} out of bounds (len {})", values.len()),
                    "Check index bounds before array access.",
                )
            })
        }
        Value::Dict(pairs) => pairs
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| {
                ValueOperationError::missing_dictionary_key(
                    format!("Key `{key}` not found in dict"),
                    "Check that the dictionary contains the key before access.",
                )
            }),
        Value::Str(text) => {
            let index = checked_index(key, "string")?;
            text.chars()
                .nth(index)
                .map(|character| Value::Str(character.to_string().into()))
                .ok_or_else(|| {
                    ValueOperationError::array_bounds(
                        format!("String index {index} out of bounds"),
                        "Check the index is in the range 0 .. Length(S) - 1.",
                    )
                })
        }
        other => Err(ValueOperationError::type_mismatch(
            format!(
                "Cannot index value of type {}; expected array, dictionary, or string",
                other.type_name()
            ),
            "Index an array or string with an integer, or a dictionary with its key type.",
        )),
    }
}

fn checked_index(key: &Value, kind: &str) -> Result<usize, ValueOperationError> {
    match key {
        Value::Integer(index) if *index >= 0 => usize::try_from(*index).map_err(|_| {
            ValueOperationError::domain(
                format!("{kind} index {index} cannot be represented on this host"),
                "Use a smaller non-negative index.",
            )
        }),
        Value::Integer(index) => Err(ValueOperationError::domain(
            format!("Negative {kind} index {index}"),
            "Indices must be non-negative integers (0-based).",
        )),
        other => Err(ValueOperationError::type_mismatch(
            format!(
                "Expected an integer {kind} index, got {}",
                other.type_name()
            ),
            format!("Use an integer index when reading a {kind}."),
        )),
    }
}
