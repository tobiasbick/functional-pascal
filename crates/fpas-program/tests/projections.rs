//! Portable images retain concrete nested fields and nominal array elements.
#![allow(clippy::expect_used, reason = "exact projection image fixtures")]

mod common;

use fpas_bytecode::{DebugProjection, DebugProjectionKind, DebugTypeId, StringId};
use fpas_program::{ProgramImage, decode, encode};

#[test]
fn concrete_record_array_and_nominal_scalar_projection_round_trip() {
    let original = common::program_image();
    let mut executable = original.executable().executable().clone();
    let scalar = DebugProjection {
        ty: DebugTypeId::new(2),
        type_name: Some(StringId::new(9)),
        kind: DebugProjectionKind::Scalar,
    };
    executable.records[0].fields[0].ty = DebugTypeId::new(5);
    executable.globals[0].ty = DebugTypeId::new(11);
    executable.globals[0].projection = Some(DebugProjection {
        ty: DebugTypeId::new(11),
        type_name: None,
        kind: DebugProjectionKind::Record(vec![DebugProjection {
            ty: DebugTypeId::new(6),
            type_name: None,
            kind: DebugProjectionKind::Array(Box::new(scalar.clone())),
        }]),
    });
    executable.functions[0].debug.bindings[0].projection = Some(scalar);
    let image = ProgramImage::new(
        original.identity().clone(),
        original.source_paths().to_vec(),
        original.source_hashes().to_vec(),
        executable.verify().expect("verified concrete projections"),
    )
    .expect("image");
    let bytes = encode(&image).expect("encode");
    let decoded = decode(&bytes).expect("decode");
    assert_eq!(decoded.executable(), image.executable());
    assert_eq!(encode(&decoded).expect("re-encode"), bytes);
}
