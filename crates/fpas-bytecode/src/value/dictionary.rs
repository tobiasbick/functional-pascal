//! Normalize dictionary construction with the same key equality used by updates.
//!
//! **Documentation:** `docs/pascal/language/types/dictionaries.md`.

use super::{SharedDict, Value};

/// Retain first key order and replace equal keys' values with their last occurrence.
pub(super) fn normalize(pairs: Vec<(Value, Value)>) -> SharedDict {
    let mut entries: Vec<(Value, Value)> = Vec::with_capacity(pairs.len());
    for (key, value) in pairs {
        if let Some((_, stored)) = entries
            .iter_mut()
            .find(|(candidate, _)| candidate.language_equal(&key))
        {
            *stored = value;
        } else {
            entries.push((key, value));
        }
    }
    entries.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_structural_keys_keep_first_order_and_last_value() {
        let first = Value::dict(vec![
            (Value::Integer(1), Value::Integer(2)),
            (Value::Integer(3), Value::Integer(4)),
        ]);
        let reordered = Value::dict(vec![
            (Value::Integer(3), Value::Integer(4)),
            (Value::Integer(1), Value::Integer(2)),
        ]);
        let dictionary = Value::dict(vec![
            (first.clone(), Value::Integer(1)),
            (Value::Integer(5), Value::Integer(2)),
            (reordered, Value::Integer(42)),
        ]);
        let Value::Dict(entries) = dictionary else {
            panic!("dictionary")
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], (first, Value::Integer(42)));
        assert_eq!(entries[1], (Value::Integer(5), Value::Integer(2)));
    }

    #[test]
    fn signed_zero_normalizes_and_nan_keys_remain_distinct() {
        let dictionary = Value::dict(vec![
            (Value::Real(-0.0), Value::Integer(1)),
            (Value::Real(0.0), Value::Integer(2)),
            (Value::Real(f64::NAN), Value::Integer(3)),
            (Value::Real(f64::NAN), Value::Integer(4)),
        ]);
        let Value::Dict(entries) = dictionary else {
            panic!("dictionary")
        };
        assert_eq!(entries.len(), 3);
        let Value::Real(key) = entries[0].0 else {
            panic!("real key")
        };
        assert_eq!(key.to_bits(), (-0.0f64).to_bits());
        assert_eq!(entries[0].1, Value::Integer(2));
    }
}
