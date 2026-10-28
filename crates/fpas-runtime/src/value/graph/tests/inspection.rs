use super::*;
use crate::release_values;

fn resource_record(slot: i64) -> Value {
    Value::Record(SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: crate::RecordTypeId::new(0),
            type_name: "Std.Net.Http.Handles.BodyStream".into(),
            fields: vec!["__Slot".into()],
        }),
        vec![Value::Integer(slot)],
    ))
}

#[test]
fn neutral_aggregates_do_not_hide_resource_records_from_observation() {
    let nested_enum = Value::Enum(SharedEnum::new(
        Arc::new(RuntimeEnumLayout {
            error: None,
            enumeration: crate::EnumTypeId::new(0),
            variant_id: crate::EnumVariantId::new(0),
            type_name: "Choice".into(),
            variant: "Some".into(),
            fields: vec!["Value".into()],
        }),
        vec![Value::result_ok(resource_record(9))],
    ));
    let value = Value::Array(
        vec![
            Value::option_some(resource_record(7)),
            Value::dict(vec![(resource_record(8), nested_enum)]),
        ]
        .into(),
    );
    assert!(value.is_ownership_neutral());
    let mut slots = Vec::new();
    let graph = inspect_values(std::slice::from_ref(&value), |value| {
        if let Value::Record(record) = value {
            assert_eq!(
                record.body().layout.type_name,
                "Std.Net.Http.Handles.BodyStream"
            );
            let Value::Integer(slot) = record.body().values[0] else {
                unreachable!()
            };
            slots.push(slot);
        }
    })
    .unwrap();
    slots.sort_unstable();
    assert_eq!(slots, [7, 8, 9]);
    assert!(graph.is_transferable());
}

#[test]
fn resource_leaves_in_keys_captures_and_bound_receivers_are_observed() {
    let function = target();
    let values = [
        Value::function(
            function.clone(),
            vec![
                Value::option_some(Value::OpaqueHandle(10)),
                resource_record(11),
            ],
        ),
        Value::bound_function(
            function.clone(),
            Value::dict(vec![(Value::OpaqueHandle(12), Value::Task(13))]),
        ),
    ];
    let mut handles = Vec::new();
    let mut tasks = Vec::new();
    let mut records = 0;
    let graph = inspect_values(&values, |value| match value {
        Value::OpaqueHandle(handle) => handles.push(*handle),
        Value::Task(task) => tasks.push(*task),
        Value::Record(_) => records += 1,
        _ => {}
    })
    .unwrap();
    handles.sort_unstable();
    assert_eq!(handles, [10, 12]);
    assert_eq!(tasks, [13]);
    assert_eq!(records, 1);
    assert!(graph.is_transferable());
    assert!(graph.executable_identities.contains(function.identity()));
}

#[test]
fn shared_allocations_and_cycles_are_observed_once() {
    let owner = MemoryDomain::new();
    let cell = owner.new_cell(Value::Unit).unwrap();
    cell.replace(
        &owner,
        Value::Array(vec![Value::Cell(cell.clone()), resource_record(1)].into()),
    )
    .unwrap();
    let roots = [Value::Cell(cell.clone()), Value::Cell(cell.clone())];
    let mut counts = [0; 3];
    let graph = inspect_values(&roots, |value| match value {
        Value::Cell(_) => counts[0] += 1,
        Value::Array(_) => counts[1] += 1,
        Value::Record(_) => counts[2] += 1,
        _ => {}
    })
    .unwrap();
    assert_eq!(counts, [1, 1, 1]);
    assert_eq!(graph.owners.len(), 1);
    assert!(graph.owners.contains(owner.id()));
    cell.replace(&owner, Value::Unit).unwrap();
}

#[test]
fn references_expose_their_cell_payloads_and_preserve_classification() {
    let owner = MemoryDomain::new();
    let value = Value::Reference(Arc::new(VariableReference {
        root: ReferenceRoot::Cell(owner.new_cell(Value::OpaqueHandle(7)).unwrap()),
        path: Vec::new(),
    }));
    let mut found = false;
    let graph = inspect_values(&[value], |value| {
        found |= matches!(value, Value::OpaqueHandle(7));
    })
    .unwrap();
    assert!(found);
    assert!(graph.has_references);
    assert!(graph.owners.contains(owner.id()));
    assert!(!graph.is_transferable());
}

#[test]
fn deep_wrapper_observation_borrows_the_original_payloads_on_a_small_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let mut value = resource_record(1);
            for _ in 0..30_000 {
                value = Value::option_some(value);
            }
            let root = Value::Array(vec![value].into());
            let Value::Array(array) = &root else {
                unreachable!()
            };
            let mut expected = Vec::new();
            let mut value = &array[0];
            while let Value::OptionSome(inner) = value {
                expected.push(std::ptr::from_ref(value) as usize);
                value = inner;
            }
            let mut observed = Vec::new();
            let mut records = 0;
            inspect_values(std::slice::from_ref(&root), |value| match value {
                Value::OptionSome(_) => observed.push(std::ptr::from_ref(value) as usize),
                Value::Record(_) => records += 1,
                _ => {}
            })
            .unwrap();
            assert_eq!(observed, expected);
            assert_eq!(records, 1);
            release_values(vec![root]);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn observer_panic_releases_detached_deep_payloads_iteratively() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let owner = MemoryDomain::new();
            let mut value = Value::Unit;
            for _ in 0..30_000 {
                value = Value::option_some(value);
            }
            let cell = owner.new_cell(Value::Array(vec![value].into())).unwrap();
            let root = Value::Cell(cell.clone());
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = inspect_values(std::slice::from_ref(&root), |value| {
                    if matches!(value, Value::Array(_)) {
                        cell.replace(&owner, Value::Unit).unwrap();
                        panic!("observer interrupted");
                    }
                });
            }));
            assert!(result.is_err());
            assert_eq!(cell.read(&owner).unwrap(), Value::Unit);
        })
        .unwrap()
        .join()
        .unwrap();
}
