//! Checked projection kinds, layouts, keys and bounds before storage replacement.

use super::*;

#[test]
fn checked_array_indices_cover_both_bounds_and_integer_extremes() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::Array(
        vec![Value::Integer(1), Value::Integer(2)].into(),
    ));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap());
    assert_eq!(
        selected.project(index(0)).unwrap().read().unwrap(),
        Value::Integer(1)
    );
    assert_eq!(
        selected.project(index(1)).unwrap().read().unwrap(),
        Value::Integer(2)
    );
    for invalid in [i64::MIN, -1, 2, i64::MAX] {
        assert_eq!(
            selected.project(index(invalid)).unwrap_err(),
            ReferencePathError::ArrayBounds {
                index: invalid,
                length: 2
            }
        );
    }
    selected.release().unwrap();
    let empty = cell(Value::Array(vec![].into()));
    let selected = SelectedReference::root(registry.reserve(&empty).unwrap());
    assert_eq!(
        selected.project(index(0)).unwrap_err(),
        ReferencePathError::ArrayBounds {
            index: 0,
            length: 0
        }
    );
}

#[test]
fn missing_dictionary_keys_fail_during_selection_without_inserting() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::dict(vec![(Value::Integer(1), Value::Integer(10))]));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap());
    assert_eq!(
        selected.project(index(2)).unwrap_err(),
        ReferencePathError::MissingKey(Value::Integer(2))
    );
    selected.release().unwrap();
    assert_eq!(
        registry.read(&root).unwrap(),
        Value::dict(vec![(Value::Integer(1), Value::Integer(10))])
    );
}

#[test]
fn strings_scalars_and_non_integer_array_indices_do_not_select_writable_storage() {
    let registry = ReferenceRegistry::default();
    for (value, step, expected, actual) in [
        (
            Value::Str("abc".into()),
            index(0),
            "array or dictionary",
            "string",
        ),
        (Value::Integer(1), field(0), "record", "integer"),
        (
            Value::Array(vec![Value::Integer(1)].into()),
            ReferenceStep::Index(Value::Boolean(true)),
            "integer array index",
            "boolean",
        ),
    ] {
        let root = cell(value);
        let selected = SelectedReference::root(registry.reserve(&root).unwrap());
        assert_eq!(
            selected.project(step).unwrap_err(),
            ReferencePathError::TypeMismatch { expected, actual }
        );
    }
}

#[test]
fn record_selection_checks_layout_and_field_slot_before_entry() {
    let registry = ReferenceRegistry::default();
    let root = cell(record(vec![Value::Integer(1)]));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap());
    assert_eq!(
        selected
            .project(ReferenceStep::Field {
                record: RecordTypeId::new(4),
                field: 0
            })
            .unwrap_err(),
        ReferencePathError::RecordLayout {
            expected: RecordTypeId::new(4),
            actual: RecordTypeId::new(3)
        }
    );
    assert_eq!(
        selected.project(field(1)).unwrap_err(),
        ReferencePathError::FieldBounds { field: 1, count: 1 }
    );
}

#[test]
fn failed_nested_write_preserves_the_whole_root() {
    let registry = ReferenceRegistry::default();
    let original = record(vec![
        Value::Array(vec![Value::Integer(1)].into()),
        Value::Integer(7),
    ]);
    let root = cell(original.clone());
    let borrowed = registry.reserve(&root).unwrap();
    borrowed.activate().unwrap();
    assert_eq!(
        borrowed.write_selected(&[field(0), index(1)], Value::Integer(42)),
        Err(ReferencePathError::ArrayBounds {
            index: 1,
            length: 1
        })
    );
    assert_eq!(borrowed.read().unwrap(), original);
}

#[test]
fn selected_paths_revalidate_changed_storage_without_panicking() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::Array(vec![Value::Integer(1)].into()));
    let borrowed = registry.reserve(&root).unwrap();
    let selected = SelectedReference::root(borrowed.clone())
        .project(index(0))
        .unwrap();
    selected.activate().unwrap();
    borrowed.write(Value::Array(vec![].into())).unwrap();
    assert_eq!(
        selected.read(),
        Err(ReferencePathError::ArrayBounds {
            index: 0,
            length: 0
        })
    );
    assert_eq!(
        selected.write(Value::Integer(42)),
        Err(ReferencePathError::ArrayBounds {
            index: 0,
            length: 0
        })
    );
    assert_eq!(borrowed.read().unwrap(), Value::Array(vec![].into()));
}
