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
                    bounds_hint(
                        index,
                        values.len(),
                        "Check index bounds before array access.",
                    ),
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
            text.char_at(index)
                .map(|character| Value::Str(character.to_string().into()))
                .ok_or_else(|| {
                    ValueOperationError::array_bounds(
                        format!(
                            "String index {index} out of bounds (len {})",
                            text.char_len()
                        ),
                        bounds_hint(
                            index,
                            text.char_len(),
                            "Check the index is in the range 0 .. S.Length() - 1.",
                        ),
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

/// Explains an index equal to the length, the usual one-based Pascal or Ada habit; otherwise
/// returns `fallback`.
///
/// **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)
pub(crate) fn bounds_hint(index: usize, length: usize, fallback: &str) -> String {
    if length > 0 && index == length {
        format!(
            "Strings and arrays start at index 0, so the last index is {} (`Length() - 1`); index {index} is one past the end.",
            length - 1
        )
    } else {
        fallback.to_owned()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn array(length: i64) -> Value {
        Value::Array((0..length).map(Value::Integer).collect::<Vec<_>>().into())
    }

    #[test]
    fn index_equal_to_the_length_names_the_last_index() {
        let error = index(&array(3), &Value::Integer(3)).expect_err("one past the end");
        assert!(error.hint.contains("the last index is 2"), "{error:?}");

        let error = index(&Value::Str("Hi".into()), &Value::Integer(2)).expect_err("past end");
        assert_eq!(error.message, "String index 2 out of bounds (len 2)");
        assert!(error.hint.contains("start at index 0"), "{error:?}");
    }

    #[test]
    fn other_out_of_range_indexes_keep_the_general_hint() {
        let error = index(&array(3), &Value::Integer(7)).expect_err("far past the end");
        assert_eq!(error.hint, "Check index bounds before array access.");
        let error = index(&array(0), &Value::Integer(0)).expect_err("empty array");
        assert_eq!(error.hint, "Check index bounds before array access.");
    }
}
