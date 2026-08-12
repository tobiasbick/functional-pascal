//! Detached array insertion and removal.

use fpas_bytecode::Value;

use super::model::ArrayTransformation;
use crate::vm::debug::{DebugErrorKind, DebugSessionError};

/// Inserts one element at a zero-based index, including immediately after the last element.
pub(in crate::vm::debug) fn insert(
    array: Value,
    index: i64,
    value: Value,
) -> Result<ArrayTransformation, DebugSessionError> {
    let Value::Array(mut elements) = array else {
        return Err(not_array());
    };
    let index = checked_index(index, elements.len().saturating_add(1), true)?;
    elements.insert(index, value);
    Ok(ArrayTransformation {
        array: Value::Array(elements),
        index,
        removed: None,
    })
}

/// Removes one element at a zero-based index.
pub(in crate::vm::debug) fn remove(
    array: Value,
    index: i64,
) -> Result<ArrayTransformation, DebugSessionError> {
    let Value::Array(mut elements) = array else {
        return Err(not_array());
    };
    let index = checked_index(index, elements.len(), false)?;
    let removed = elements.remove(index);
    Ok(ArrayTransformation {
        array: Value::Array(elements),
        index,
        removed: Some(removed),
    })
}

fn checked_index(
    index: i64,
    upper_bound: usize,
    insertion: bool,
) -> Result<usize, DebugSessionError> {
    let converted = usize::try_from(index).ok();
    if let Some(index) = converted.filter(|index| *index < upper_bound) {
        return Ok(index);
    }
    let range = if insertion {
        format!("0..={}", upper_bound.saturating_sub(1))
    } else if upper_bound == 0 {
        "empty".to_string()
    } else {
        format!("0..{}", upper_bound - 1)
    };
    Err(DebugSessionError {
        kind: DebugErrorKind::SequenceIndexOutOfBounds,
        message: format!("debug array index {index} is outside the permitted range {range}"),
        hint: if insertion {
            "Use an index from zero through the current array length.".to_string()
        } else {
            "Use an index returned by Variables for the current array.".to_string()
        },
    })
}

fn not_array() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: "debug array mutation target is not an array".to_string(),
        hint: "Select a mutable target whose complete value is an array.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_bytecode::SharedArray;

    fn array(values: &[i64]) -> Value {
        Value::Array(SharedArray::from(
            values
                .iter()
                .copied()
                .map(Value::Integer)
                .collect::<Vec<_>>(),
        ))
    }

    #[test]
    fn insert_accepts_first_middle_and_end_without_changing_original() {
        for (index, expected) in [(0, vec![9, 1, 2]), (1, vec![1, 9, 2]), (2, vec![1, 2, 9])] {
            let original = array(&[1, 2]);
            let changed = insert(original.clone(), index, Value::Integer(9)).expect("insert");
            assert_eq!(original, array(&[1, 2]));
            assert_eq!(changed.array, array(&expected));
            assert_eq!(changed.index, index as usize);
        }
    }

    #[test]
    fn insert_accepts_empty_array_and_remove_accepts_final_element() {
        let inserted = insert(array(&[]), 0, Value::Integer(7)).expect("insert into empty");
        assert_eq!(inserted.array, array(&[7]));
        let removed = remove(inserted.array, 0).expect("remove final element");
        assert_eq!(removed.array, array(&[]));
        assert_eq!(removed.removed, Some(Value::Integer(7)));
    }

    #[test]
    fn remove_accepts_first_middle_and_last() {
        for (index, expected, removed) in
            [(0, vec![2, 3], 1), (1, vec![1, 3], 2), (2, vec![1, 2], 3)]
        {
            let changed = remove(array(&[1, 2, 3]), index).expect("remove");
            assert_eq!(changed.array, array(&expected));
            assert_eq!(changed.removed, Some(Value::Integer(removed)));
        }
    }

    #[test]
    fn invalid_indices_are_stable_errors() {
        for error in [
            insert(array(&[1]), -1, Value::Integer(2)).expect_err("negative"),
            insert(array(&[1]), 2, Value::Integer(2)).expect_err("large"),
            remove(array(&[]), 0).expect_err("empty"),
        ] {
            assert_eq!(error.kind, DebugErrorKind::SequenceIndexOutOfBounds);
        }
    }
}
