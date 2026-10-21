use super::*;
use fpas_bytecode::{EnumTypeId, EnumVariantId, RuntimeEnumLayout, SharedEnum};

#[test]
fn selecting_children_preserves_origin_without_exposing_resource_siblings() {
    let image = empty_image();
    let origin = HostValues::new(MemoryDomain::new());
    let foreign = HostValues::new(MemoryDomain::new());
    let value = exported(
        &origin,
        Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                enumeration: EnumTypeId::new(2),
                variant_id: EnumVariantId::new(3),
                type_name: "Envelope".into(),
                variant: "Value".into(),
                error: None,
                fields: vec!["data".into(), "handle".into()],
            }),
            vec![Value::Integer(42), Value::OpaqueHandle(1)],
        )),
    );
    assert_eq!(value.try_to_data(), Err(HostValueError::SessionBound));
    let info = value.enum_info().expect("enum metadata");
    assert_eq!(info.enumeration, EnumTypeId::new(2));
    assert_eq!(info.fields, ["data", "handle"]);
    let data = value
        .child(HostChild::Field("DATA"))
        .expect("child")
        .expect("data");
    assert_eq!(data.try_to_data().expect("pure child"), Value::Integer(42));
    assert!(foreign.admit(vec![data.into()], &image).is_ok());
    let handle = value
        .child(HostChild::Index(1))
        .expect("child")
        .expect("handle");
    assert_eq!(handle.try_to_data(), Err(HostValueError::SessionBound));
    assert!(origin.admit(vec![handle.clone().into()], &image).is_ok());
    assert_eq!(
        foreign
            .admit(vec![handle.into()], &image)
            .expect_err("child retains origin")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert!(
        value
            .child(HostChild::Index(2))
            .expect("missing child")
            .is_none()
    );
}

#[test]
fn dictionary_key_value_and_payload_selection_remain_sealed() {
    let context = HostValues::new(MemoryDomain::new());
    let value = exported(
        &context,
        Value::dict(vec![(
            Value::Integer(3),
            Value::option_some(Value::Integer(5)),
        )]),
    );
    assert_eq!(
        value
            .child(HostChild::DictKey(0))
            .expect("key")
            .expect("key exists")
            .try_to_data()
            .expect("pure key"),
        Value::Integer(3)
    );
    assert_eq!(
        value
            .child(HostChild::DictValue(0))
            .expect("value")
            .expect("value exists")
            .child(HostChild::Payload)
            .expect("payload")
            .expect("some")
            .try_to_data()
            .expect("pure payload"),
        Value::Integer(5)
    );
}
