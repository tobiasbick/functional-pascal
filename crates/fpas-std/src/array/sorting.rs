//! Stable scalar and cached-key sorting.
//! Documentation: `docs/pascal/language/types/array/basics.md`.

use std::cmp::Ordering;

use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

use crate::error::{StdError, std_runtime_error};

/// Sort values ascending after checking their scalar ordering and finite reals.
pub(super) fn sort_values(values: &mut [Value], location: SourceLocation) -> Result<(), StdError> {
    validate_ordering(values.iter(), "Sort", location)?;
    values.sort_by(compare_validated);
    Ok(())
}

/// Stably sort cached `(key, element)` pairs and return their elements.
///
/// The VM evaluates keys once in input order before calling this algorithm.
/// Keys obey the same Comparable ordering and finite-real rules as `Sort`.
pub fn sort_by_cached_keys(
    mut entries: Vec<(Value, Value)>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    validate_ordering(entries.iter().map(|(key, _)| key), "SortBy", location)?;
    entries.sort_by(|(left, _), (right, _)| compare_validated(left, right));
    Ok(Value::Array(
        entries.into_iter().map(|(_, value)| value).collect(),
    ))
}

fn validate_ordering<'a>(
    mut values: impl Iterator<Item = &'a Value>,
    operation: &str,
    location: SourceLocation,
) -> Result<(), StdError> {
    let Some(first) = values.next() else {
        return Ok(());
    };
    for value in std::iter::once(first).chain(values) {
        if let Value::Real(real) = value {
            fpas_numeric::real::finite(*real).map_err(|error| {
                std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    error.message(),
                    error.hint(),
                    location,
                )
            })?;
        }
        if value.scalar_ordering(first).is_none() {
            return Err(std_runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("{operation} cannot order {} values", value.type_name()),
                "Use homogeneous Comparable elements for Sort or Comparable keys for SortBy.",
                location,
            ));
        }
    }
    Ok(())
}

#[expect(
    clippy::expect_used,
    reason = "scalar ordering was validated before sorting"
)]
fn compare_validated(left: &Value, right: &Value) -> Ordering {
    left.scalar_ordering(right)
        .expect("validated scalar ordering")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_keys_validate_even_singletons_and_heterogeneous_runtime_values() {
        for keys in [
            vec![Value::Real(f64::NAN)],
            vec![Value::Real(f64::INFINITY)],
            vec![Value::Real(f64::NEG_INFINITY)],
            vec![Value::Integer(1), Value::Boolean(true)],
            vec![Value::OptionNone],
        ] {
            let expected = if matches!(keys[0], Value::Real(_)) {
                RUNTIME_NUMERIC_DOMAIN_ERROR
            } else {
                RUNTIME_VM_OPERAND_TYPE_MISMATCH
            };
            let entries = keys.into_iter().map(|key| (key, Value::Unit)).collect();
            let error = sort_by_cached_keys(entries, SourceLocation::new(1, 1)).unwrap_err();
            assert_eq!(error.code, expected);
        }
    }
}
