//! Selected-reference bounds, authority, copy isolation, and immediate updates.

use std::sync::{Arc, Mutex};

use super::*;

mod authority;
mod validation;
use crate::{ReferenceRegistry, RuntimeRecordLayout, SelectedReference, SharedRecord};

fn cell(value: Value) -> Arc<Mutex<Value>> {
    Arc::new(Mutex::new(value))
}

fn record(values: Vec<Value>) -> Value {
    Value::Record(SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(3),
            type_name: "Holder".to_string(),
            fields: (0..values.len())
                .map(|index| format!("Field{index}"))
                .collect(),
        }),
        values,
    ))
}

fn index(index: i64) -> ReferenceStep {
    ReferenceStep::Index(Value::Integer(index))
}

fn field(field: usize) -> ReferenceStep {
    ReferenceStep::Field {
        record: RecordTypeId::new(3),
        field,
    }
}

#[test]
fn selected_root_replaces_immediately_and_release_does_not_undo_the_write() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::Integer(1));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap());
    selected.activate().unwrap();
    selected.write(Value::Integer(9)).unwrap();
    assert_eq!(selected.read().unwrap(), Value::Integer(9));
    selected.release().unwrap();
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(9));
}

#[test]
fn nested_record_dictionary_array_updates_preserve_siblings_and_snapshots() {
    let registry = ReferenceRegistry::default();
    let original = record(vec![
        Value::dict(vec![(
            Value::Str("key".into()),
            Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
        )]),
        Value::Integer(7),
    ]);
    let root = cell(original.clone());
    let selected = SelectedReference::root(registry.reserve(&root).unwrap())
        .project(field(0))
        .unwrap()
        .project(ReferenceStep::Index(Value::Str("key".into())))
        .unwrap()
        .project(index(1))
        .unwrap();
    let snapshot = registry.read(&root).unwrap();
    selected.activate().unwrap();
    selected.write(Value::Integer(42)).unwrap();
    selected.release().unwrap();
    let updated = registry.read(&root).unwrap();
    assert_eq!(snapshot, original);
    assert_eq!(
        read(
            &updated,
            &[
                field(0),
                ReferenceStep::Index(Value::Str("key".into())),
                index(0)
            ]
        )
        .unwrap(),
        &Value::Integer(1)
    );
    assert_eq!(
        read(
            &updated,
            &[
                field(0),
                ReferenceStep::Index(Value::Str("key".into())),
                index(1)
            ]
        )
        .unwrap(),
        &Value::Integer(42)
    );
    assert_eq!(read(&updated, &[field(1)]).unwrap(), &Value::Integer(7));
}

#[test]
fn dictionary_selection_uses_structural_language_key_equality() {
    let registry = ReferenceRegistry::default();
    let key = Value::dict(vec![
        (Value::Integer(1), Value::Integer(10)),
        (Value::Integer(2), Value::Integer(20)),
    ]);
    let reordered = Value::dict(vec![
        (Value::Integer(2), Value::Integer(20)),
        (Value::Integer(1), Value::Integer(10)),
    ]);
    let root = cell(Value::dict(vec![(key, Value::Integer(1))]));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap())
        .project(ReferenceStep::Index(reordered))
        .unwrap();
    selected.activate().unwrap();
    selected.write(Value::Integer(42)).unwrap();
    assert_eq!(selected.read().unwrap(), Value::Integer(42));
    selected.release().unwrap();
    let Value::Dict(entries) = registry.read(&root).unwrap() else {
        panic!("dictionary")
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1, Value::Integer(42));
}

#[test]
fn stored_dictionary_key_remains_a_snapshot_of_the_evaluated_key() {
    let registry = ReferenceRegistry::default();
    let mut evaluated = Value::Array(vec![Value::Integer(1)].into());
    let root = cell(Value::dict(vec![(evaluated.clone(), Value::Integer(10))]));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap())
        .project(ReferenceStep::Index(evaluated.clone()))
        .unwrap();
    let Value::Array(values) = &mut evaluated else {
        panic!("array key")
    };
    values[0] = Value::Integer(2);
    selected.activate().unwrap();
    selected.write(Value::Integer(42)).unwrap();
    assert_eq!(selected.read().unwrap(), Value::Integer(42));
}

#[test]
fn selected_resource_and_closure_snapshots_preserve_declared_identity() {
    let registry = ReferenceRegistry::default();
    let capture = cell(Value::Integer(1));
    let action = Value::function(
        crate::FunctionId::new(1),
        "Action",
        vec![Value::Cell(Arc::clone(&capture))],
    );
    let root = cell(Value::Array(
        vec![Value::OpaqueHandle(7), action.clone()].into(),
    ));
    let borrowed = SelectedReference::root(registry.reserve(&root).unwrap());
    let resource = borrowed.project(index(0)).unwrap().read().unwrap();
    let closure = borrowed.project(index(1)).unwrap().read().unwrap();
    assert_eq!(resource, Value::OpaqueHandle(7));
    let (Value::Function(original), Value::Function(snapshot)) = (action, closure) else {
        panic!("closure")
    };
    let (Value::Cell(original), Value::Cell(snapshot)) =
        (&original.captures[0], &snapshot.captures[0])
    else {
        panic!("capture cell")
    };
    assert!(Arc::ptr_eq(original, snapshot));
}
