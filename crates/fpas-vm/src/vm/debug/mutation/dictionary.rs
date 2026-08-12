//! Pure copy-on-write dictionary structure transformations.

use fpas_bytecode::Value;

use super::super::types::{DebugErrorKind, DebugSessionError};

#[derive(Debug)]
/// Detached dictionary value and optional operation metadata prepared for commit.
pub(in crate::vm::debug) struct DictionaryTransformation {
    pub dictionary: Value,
    pub removed: Option<Value>,
    pub old_key: Option<Value>,
    pub new_key: Option<Value>,
}

/// Appends one missing key/value pair to a detached dictionary value.
pub(in crate::vm::debug) fn insert(
    dictionary: Value,
    key: Value,
    value: Value,
) -> Result<DictionaryTransformation, DebugSessionError> {
    let Value::Dict(mut entries) = dictionary else {
        return Err(not_dictionary());
    };
    if entries.iter().any(|(candidate, _)| candidate == &key) {
        return Err(key_exists(&key));
    }
    entries.push((key, value));
    Ok(DictionaryTransformation {
        dictionary: Value::Dict(entries),
        removed: None,
        old_key: None,
        new_key: None,
    })
}

/// Removes one existing key/value pair from a detached dictionary value.
pub(in crate::vm::debug) fn remove(
    dictionary: Value,
    key: &Value,
) -> Result<DictionaryTransformation, DebugSessionError> {
    let Value::Dict(mut entries) = dictionary else {
        return Err(not_dictionary());
    };
    let Some(index) = entries.iter().position(|(candidate, _)| candidate == key) else {
        return Err(key_missing(key));
    };
    let (_, removed) = entries.remove(index);
    Ok(DictionaryTransformation {
        dictionary: Value::Dict(entries),
        removed: Some(removed),
        old_key: None,
        new_key: None,
    })
}

/// Replaces one existing key while preserving its detached value and position.
pub(in crate::vm::debug) fn replace_key(
    dictionary: Value,
    old_key: &Value,
    new_key: Value,
) -> Result<DictionaryTransformation, DebugSessionError> {
    let Value::Dict(mut entries) = dictionary else {
        return Err(not_dictionary());
    };
    let Some(index) = entries
        .iter()
        .position(|(candidate, _)| candidate == old_key)
    else {
        return Err(key_missing(old_key));
    };
    if old_key == &new_key {
        return Err(DebugSessionError {
            kind: DebugErrorKind::DictionaryKeyUnchanged,
            message: format!("debug dictionary key `{old_key}` is unchanged"),
            hint: "Use a different missing key or leave the dictionary unchanged.".to_string(),
        });
    }
    if entries.iter().any(|(candidate, _)| candidate == &new_key) {
        return Err(key_exists(&new_key));
    }
    entries[index].0 = new_key.clone();
    Ok(DictionaryTransformation {
        dictionary: Value::Dict(entries),
        removed: None,
        old_key: Some(old_key.clone()),
        new_key: Some(new_key),
    })
}

fn not_dictionary() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: "debug dictionary mutation target is not a dictionary".to_string(),
        hint: "Select a mutable target whose complete value is `dict of K to V`.".to_string(),
    }
}

fn key_exists(key: &Value) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::DictionaryKeyExists,
        message: format!("debug dictionary key `{key}` already exists"),
        hint: "Choose a missing key, or use setExpression to replace the existing value."
            .to_string(),
    }
}

fn key_missing(key: &Value) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::DictionaryKeyMissing,
        message: format!("debug dictionary key `{key}` does not exist"),
        hint: "Choose a key returned by Variables for the current dictionary.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary() -> Value {
        Value::dict(vec![
            (Value::Integer(1), Value::Integer(10)),
            (Value::Integer(2), Value::Integer(20)),
            (Value::Integer(3), Value::Integer(30)),
        ])
    }

    fn pairs(value: &Value) -> &[(Value, Value)] {
        let Value::Dict(entries) = value else {
            panic!("expected dictionary")
        };
        entries
    }

    #[test]
    fn insert_appends_without_changing_the_original() {
        let original = dictionary();
        let changed = insert(original.clone(), Value::Integer(4), Value::Integer(40))
            .expect("insert missing key");
        assert_eq!(pairs(&original).len(), 3);
        assert_eq!(
            pairs(&changed.dictionary)[3],
            (Value::Integer(4), Value::Integer(40))
        );
    }

    #[test]
    fn remove_first_middle_and_last_preserve_remaining_order() {
        for (removed_key, remaining_keys, removed_value) in [
            (1, vec![2, 3], 10),
            (2, vec![1, 3], 20),
            (3, vec![1, 2], 30),
        ] {
            let changed =
                remove(dictionary(), &Value::Integer(removed_key)).expect("remove existing key");
            assert_eq!(
                pairs(&changed.dictionary)
                    .iter()
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>(),
                remaining_keys
                    .into_iter()
                    .map(Value::Integer)
                    .collect::<Vec<_>>()
            );
            assert_eq!(changed.removed, Some(Value::Integer(removed_value)));
        }
    }

    #[test]
    fn replace_key_preserves_value_and_position() {
        let changed = replace_key(dictionary(), &Value::Integer(2), Value::Integer(4))
            .expect("replace existing key");
        assert_eq!(
            pairs(&changed.dictionary)[1],
            (Value::Integer(4), Value::Integer(20))
        );
        assert_eq!(changed.old_key, Some(Value::Integer(2)));
        assert_eq!(changed.new_key, Some(Value::Integer(4)));
    }

    #[test]
    fn collisions_missing_keys_and_no_ops_are_distinct() {
        assert_eq!(
            insert(dictionary(), Value::Integer(2), Value::Integer(99))
                .expect_err("insert collision")
                .kind,
            DebugErrorKind::DictionaryKeyExists
        );
        assert_eq!(
            remove(dictionary(), &Value::Integer(9))
                .expect_err("missing removal")
                .kind,
            DebugErrorKind::DictionaryKeyMissing
        );
        assert_eq!(
            replace_key(dictionary(), &Value::Integer(2), Value::Integer(2))
                .expect_err("unchanged key")
                .kind,
            DebugErrorKind::DictionaryKeyUnchanged
        );
        assert_eq!(
            replace_key(dictionary(), &Value::Integer(2), Value::Integer(3))
                .expect_err("replacement collision")
                .kind,
            DebugErrorKind::DictionaryKeyExists
        );
    }
}
