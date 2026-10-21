use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::{
    EnumTypeId, EnumVariantId, MemoryDomain, RecordTypeId, ReferenceRoot, RuntimeEnumLayout,
    RuntimeRecordLayout, SharedArray, SharedDict, SharedEnum, SharedRecord, Value,
    VariableReference, classify_values,
};

fn record(value: Value) -> SharedRecord {
    SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(0),
            type_name: "Holder".into(),
            fields: vec!["Value".into()],
        }),
        vec![value],
    )
}

fn enumeration(value: Value) -> SharedEnum {
    SharedEnum::new(
        Arc::new(RuntimeEnumLayout {
            error: None,
            enumeration: EnumTypeId::new(0),
            variant_id: EnumVariantId::new(0),
            type_name: "Choice".into(),
            variant: "Some".into(),
            fields: vec!["Value".into()],
        }),
        vec![value],
    )
}

#[test]
fn nested_immutable_aggregates_and_wrappers_preserve_the_summary() {
    let mut value = Value::Integer(1);
    for _ in 0..8 {
        value = Value::Array(
            vec![Value::Record(record(Value::option_some(Value::dict(
                vec![(
                    Value::Str("entry".into()),
                    Value::Enum(enumeration(Value::result_ok(value))),
                )],
            ))))]
            .into(),
        );
        assert!(value.is_ownership_neutral());
    }
    let Value::Array(array) = &value else {
        unreachable!();
    };
    assert!(!array.is_plain_data());
    assert!(classify_values(&[value]).unwrap().is_transferable());
}

#[test]
fn resource_shaped_records_are_neutral_without_becoming_plain_data() {
    let handle = SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(0),
            type_name: "HttpBodyStream".into(),
            fields: vec!["__Slot".into()],
        }),
        vec![Value::Integer(7)],
    );
    let array = SharedArray::from(vec![Value::Record(handle)]);
    assert!(array.is_ownership_neutral());
    assert!(!array.is_plain_data());
}

#[test]
fn local_or_capability_leaves_in_any_position_prevent_a_neutral_summary() {
    let owner = MemoryDomain::new();
    for value in [
        Value::Cell(owner.new_cell(Value::Unit).unwrap()),
        Value::Reference(Arc::new(VariableReference {
            root: ReferenceRoot::Global {
                slot: 0,
                owner: owner.id().clone(),
            },
            path: Vec::new(),
        })),
        Value::Task(3),
        Value::OpaqueHandle(4),
    ] {
        let wrapped = Value::option_some(Value::result_error(value));
        assert!(!wrapped.is_ownership_neutral());
        assert!(!record(wrapped.clone()).is_ownership_neutral());
        assert!(!enumeration(wrapped.clone()).is_ownership_neutral());
        assert!(!SharedArray::from(vec![wrapped.clone()]).is_ownership_neutral());
        assert!(!SharedDict::from(vec![(wrapped.clone(), Value::Unit)]).is_ownership_neutral());
        assert!(!SharedDict::from(vec![(Value::Unit, wrapped)]).is_ownership_neutral());
    }
}

#[test]
fn nested_updates_preserve_the_proof_and_invalidate_it_for_inserted_cells() {
    let original = SharedArray::from(vec![Value::Record(record(Value::dict(vec![(
        Value::Unit,
        Value::Enum(enumeration(Value::Integer(1))),
    )])))]);
    let mut updated = original.clone();
    let edit = |root: &mut Value, replacement| {
        let Value::Record(record) = root else {
            unreachable!()
        };
        record.update_field(0, |child| {
            let Value::Dict(dictionary) = child else {
                unreachable!()
            };
            dictionary.update_value(0, |child| {
                let Value::Enum(enumeration) = child else {
                    unreachable!()
                };
                enumeration.update_field(0, |child| *child = replacement);
            });
        });
    };
    updated.update_element(0, |child| edit(child, Value::Integer(2)));
    assert!(updated.is_ownership_neutral());
    assert!(original.is_ownership_neutral());
    assert_ne!(
        Value::Array(original.clone()),
        Value::Array(updated.clone())
    );

    let owner = MemoryDomain::new();
    updated.update_element(0, |child| {
        edit(child, Value::Cell(owner.new_cell(Value::Unit).unwrap()));
    });
    assert!(!updated.is_ownership_neutral());
    assert!(original.is_ownership_neutral());
    let classification = classify_values(&[Value::Array(updated)]).unwrap();
    assert!(classification.owners.contains(owner.id()));
}

#[test]
fn specialized_replacements_preserve_summaries_only_for_neutral_inputs() {
    let mut record_value = record(Value::Unit);
    assert_eq!(
        record_value.replace_field(0, Value::Integer(2)),
        Value::Unit
    );
    assert!(record_value.is_ownership_neutral());
    record_value.replace_field(0, Value::Task(1));
    assert!(!record_value.is_ownership_neutral());

    let mut enumeration = enumeration(Value::Unit);
    assert_eq!(enumeration.replace_field(0, Value::Integer(2)), Value::Unit);
    assert!(enumeration.is_ownership_neutral());
    enumeration.replace_field(0, Value::OpaqueHandle(1));
    assert!(!enumeration.is_ownership_neutral());

    let mut dictionary = SharedDict::from(vec![(Value::Unit, Value::Unit)]);
    assert_eq!(dictionary.replace_value(0, Value::Integer(2)), Value::Unit);
    dictionary.push((Value::Integer(1), Value::Record(record(Value::Unit))));
    assert!(dictionary.is_ownership_neutral());
    dictionary.push((Value::OpaqueHandle(1), Value::Unit));
    assert!(!dictionary.is_ownership_neutral());
    dictionary.replace_value(0, Value::Unit);
    assert!(!dictionary.is_ownership_neutral());
}

#[test]
fn raw_mutation_invalidates_every_aggregate_without_affecting_shared_originals() {
    let original_record = record(Value::Unit);
    let mut changed_record = original_record.clone();
    let _ = changed_record.values_mut();
    assert!(!changed_record.is_ownership_neutral());
    assert!(original_record.is_ownership_neutral());

    let original_enum = enumeration(Value::Unit);
    let mut changed_enum = original_enum.clone();
    let _ = changed_enum.values_mut();
    assert!(!changed_enum.is_ownership_neutral());
    assert!(original_enum.is_ownership_neutral());

    let original_dict = SharedDict::from(vec![(Value::Unit, Value::Unit)]);
    let mut changed_dict = original_dict.clone();
    changed_dict[0].0 = Value::Task(1);
    assert!(!changed_dict.is_ownership_neutral());
    assert!(original_dict.is_ownership_neutral());

    let original_array = SharedArray::from(vec![Value::Unit]);
    let mut changed_array = original_array.clone();
    changed_array[0] = Value::Task(1);
    assert!(!changed_array.is_ownership_neutral());
    assert!(original_array.is_ownership_neutral());
}

#[test]
fn panicking_callbacks_never_leave_stale_aggregate_summaries() {
    let mut array = SharedArray::from(vec![Value::Unit]);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.update_element(0, |_| panic!("aborted update"));
        }))
        .is_err()
    );
    assert!(!array.is_plain_data());
    assert!(!array.is_ownership_neutral());

    let mut record = record(Value::Unit);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            record.update_field(0, |_| panic!("aborted update"));
        }))
        .is_err()
    );
    assert!(!record.is_ownership_neutral());

    let mut enumeration = enumeration(Value::Unit);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            enumeration.update_field(0, |_| panic!("aborted update"));
        }))
        .is_err()
    );
    assert!(!enumeration.is_ownership_neutral());

    let mut dictionary = SharedDict::from(vec![(Value::Unit, Value::Unit)]);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            dictionary.update_value(0, |_| panic!("aborted update"));
        }))
        .is_err()
    );
    assert!(!dictionary.is_ownership_neutral());
}

#[test]
fn wrapper_chains_and_deep_shared_aggregates_do_not_require_recursive_summaries() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let owner = MemoryDomain::new();
            let mut value = Value::Unit;
            for _ in 0..30_000 {
                value = Value::option_some(value);
            }
            assert!(value.is_ownership_neutral());
            let cell = owner.new_cell(Value::Array(vec![value].into())).unwrap();
            drop(cell);

            let mut value = Value::Unit;
            for _ in 0..10_000 {
                value = Value::Array(vec![Value::option_some(value)].into());
            }
            assert!(value.is_ownership_neutral());
            drop(owner.new_cell(value).unwrap());
        })
        .unwrap()
        .join()
        .unwrap();
}
