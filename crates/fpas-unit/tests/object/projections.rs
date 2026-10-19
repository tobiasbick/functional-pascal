//! Object projection preservation and rejection before unbounded allocation.

use fpas_bytecode::{DebugProjection, DebugProjectionKind, DebugType, DebugTypeId, StringId};
use fpas_unit::object::{
    ObjectDebugProjection, ObjectDebugProjectionKind, RelocatableObject, decode_object,
    encode_object,
};

fn object() -> RelocatableObject {
    let mut executable = super::candidate();
    executable.debug_types.push(DebugType::Integer);
    let projection = DebugProjection {
        ty: DebugTypeId::new(4),
        type_name: Some(StringId::new(2)),
        kind: DebugProjectionKind::Scalar,
    };
    executable.globals[0].projection = Some(projection.clone());
    executable.functions[0].debug.bindings[0].projection = Some(projection);
    RelocatableObject::from_executable("demo", executable.verify().expect("verified fixture"))
        .expect("object")
}

#[test]
fn source_checked_global_and_binding_projections_round_trip() {
    let object = object();
    let bytes = encode_object(&object).expect("encode projections");
    let decoded = decode_object(&bytes).expect("decode projections");
    assert_eq!(decoded, object);
    let projection = decoded.globals[0]
        .projection
        .as_ref()
        .expect("global projection");
    assert_eq!(projection.ty, 4);
    assert_eq!(projection.type_name.as_deref(), Some("value"));
    assert_eq!(projection.kind, ObjectDebugProjectionKind::Scalar);
}

#[test]
fn decoded_projection_depth_node_count_and_names_are_bounded() {
    let leaf = r#"{"ty":0,"type_name":null,"kind":"Unsupported"}"#;
    for count in [4095, 4096] {
        let input = format!(
            r#"{{"ty":0,"type_name":null,"kind":{{"Record":[{}]}}}}"#,
            vec![leaf; count].join(",")
        );
        let parsed = serde_json::from_str::<ObjectDebugProjection>(&input);
        assert_eq!(parsed.is_ok(), count == 4095, "node boundary");
    }
    for depth in [32, 33] {
        let mut input = leaf.to_owned();
        for _ in 0..depth {
            input = format!(r#"{{"ty":0,"type_name":null,"kind":{{"Array":{input}}}}}"#);
        }
        assert_eq!(
            serde_json::from_str::<ObjectDebugProjection>(&input).is_ok(),
            depth == 32,
            "depth boundary"
        );
    }
    for length in [65_536, 65_537] {
        let input = format!(
            r#"{{"ty":0,"type_name":"{}","kind":"Unsupported"}}"#,
            "n".repeat(length)
        );
        assert_eq!(
            serde_json::from_str::<ObjectDebugProjection>(&input).is_ok(),
            length == 65_536,
            "name byte boundary"
        );
    }
}

#[test]
fn object_validator_rejects_scalar_projection_claiming_record_storage() {
    let mut object = object();
    object.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .ty = 1;
    assert!(encode_object(&object).is_err());
}

#[test]
fn operational_tail_round_trips_and_rejects_semantic_refinement_or_names() {
    let mut object = object();
    object.globals[0].projection = Some(ObjectDebugProjection {
        ty: 0,
        type_name: None,
        kind: ObjectDebugProjectionKind::Operational,
    });
    let bytes = encode_object(&object).expect("valid operational tail");
    assert_eq!(decode_object(&bytes).expect("decoded tail"), object);
    object.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .ty = 4;
    assert!(
        encode_object(&object).is_err(),
        "Operational cannot refine Dynamic"
    );
    object.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .ty = 0;
    object.globals[0]
        .projection
        .as_mut()
        .expect("projection")
        .type_name = Some("Dollars".into());
    assert!(
        encode_object(&object).is_err(),
        "Operational cannot invent nominal names"
    );
}
