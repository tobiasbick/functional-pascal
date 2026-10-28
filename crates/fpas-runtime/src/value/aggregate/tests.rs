use super::*;

#[test]
fn cloned_dict_detaches_on_mutation() {
    let original = SharedDict::from(vec![(Value::Integer(1), Value::Integer(2))]);
    let mut updated = original.clone();
    updated[0].1 = Value::Integer(3);
    assert_eq!(original[0].1, Value::Integer(2));
    assert_eq!(updated[0].1, Value::Integer(3));
}

#[test]
fn cloned_record_detaches_on_mutation() {
    let layout = Arc::new(RuntimeRecordLayout {
        record: RecordTypeId::new(0),
        type_name: "Point".to_string(),
        fields: vec!["x".to_string()],
    });
    let original = SharedRecord::new(layout, vec![Value::Integer(1)]);
    let mut updated = original.clone();
    updated.values_mut()[0] = Value::Integer(2);
    assert_eq!(original.body().values[0], Value::Integer(1));
    assert_eq!(updated.body().values[0], Value::Integer(2));
}

#[test]
fn cloned_enum_detaches_on_mutation() {
    let layout = Arc::new(RuntimeEnumLayout {
        error: None,
        enumeration: EnumTypeId::new(0),
        variant_id: EnumVariantId::new(0),
        type_name: "Choice".to_string(),
        variant: "Count".to_string(),
        fields: vec!["Value".to_string()],
    });
    let original = SharedEnum::new(layout, vec![Value::Integer(1)]);
    let mut updated = original.clone();
    updated.values_mut()[0] = Value::Integer(2);
    assert_eq!(original.body().values[0], Value::Integer(1));
    assert_eq!(updated.body().values[0], Value::Integer(2));
}

#[test]
fn ordinary_aggregate_drop_releases_deep_graphs_without_recursive_destructors() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let record = Arc::new(RuntimeRecordLayout {
                record: RecordTypeId::new(0),
                type_name: "Holder".into(),
                fields: vec!["Value".into()],
            });
            let enumeration = Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: EnumTypeId::new(0),
                variant_id: EnumVariantId::new(0),
                type_name: "Choice".into(),
                variant: "Some".into(),
                fields: vec!["Value".into()],
            });
            let owner = crate::MemoryDomain::new();
            for root_kind in 0..4 {
                let cell = owner.new_cell(Value::Unit).unwrap();
                let weak = cell.downgrade();
                let mut value = Value::Cell(cell);
                for depth in 0..20_000 {
                    value = match (depth + root_kind) % 4 {
                        0 => Value::Array(vec![value].into()),
                        1 => Value::Record(SharedRecord::new(record.clone(), vec![value])),
                        2 => Value::Enum(SharedEnum::new(enumeration.clone(), vec![value])),
                        _ if (depth / 4) % 2 == 0 => Value::dict(vec![(value, Value::Unit)]),
                        _ => Value::dict(vec![(Value::Unit, value)]),
                    };
                }
                drop(value);
                assert!(weak.upgrade().is_none());
                assert_eq!(owner.registered_count(), 0);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
