#![allow(
    clippy::expect_used,
    reason = "object fixtures use expect for compact round-trip assertions"
)]

use std::collections::BTreeMap;

use fpas_bytecode::Op;
use fpas_unit::object::{
    ObjectConstant, ObjectFunction, ObjectLocation, RelocatableObject, collect_relocations,
    decode_object, encode_object,
};

fn object_with_all_relocation_shapes() -> RelocatableObject {
    let code = vec![
        Op::Constant(0),
        Op::GetGlobal(1),
        Op::SetGlobal(1),
        Op::GlobalIndexSet(1, 2),
        Op::Call(1, 0),
        Op::MakeClosure(1, 0),
        Op::MakeRecord(1, 0),
        Op::FieldGet(1),
        Op::FieldSet(1),
        Op::MakeEnum(1, 2, 0),
        Op::IsVariant(1, 2),
        Op::Jump(12),
        Op::Halt,
    ];
    RelocatableObject {
        owner: "demo.unit".to_string(),
        constants: vec![
            ObjectConstant::Integer(42),
            ObjectConstant::String("demo.unit.name".to_string()),
            ObjectConstant::String("Variant".to_string()),
        ],
        locations: vec![
            ObjectLocation {
                line: 1,
                column: 1,
                source_id: 2,
            };
            code.len()
        ],
        functions: BTreeMap::from([(
            "demo.unit.run".to_string(),
            ObjectFunction {
                code_start: 0,
                arity: 0,
            },
        )]),
        definitions: Vec::new(),
        imports: Vec::new(),
        relocations: collect_relocations(&code),
        code,
    }
}

#[test]
fn relocatable_object_round_trip_preserves_operands_and_locations() {
    let object = object_with_all_relocation_shapes();
    let bytes = encode_object(&object).expect("object encoding");
    assert_eq!(decode_object(&bytes).expect("object decoding"), object);
}

#[test]
fn relocation_discovery_is_deterministic_and_covers_both_enum_constants() {
    let object = object_with_all_relocation_shapes();
    assert_eq!(object.relocations, collect_relocations(&object.code));
    let enum_relocations = object
        .relocations
        .iter()
        .filter(|relocation| relocation.instruction == 9)
        .count();
    assert_eq!(enum_relocations, 2);
}

#[test]
fn validation_rejects_missing_or_extra_relocations() {
    let mut object = object_with_all_relocation_shapes();
    object.relocations.pop();
    assert!(object.validate().is_err());
}

#[test]
fn validation_rejects_out_of_range_constant_operands() {
    let mut object = object_with_all_relocation_shapes();
    object.code[0] = Op::Constant(99);
    object.relocations = collect_relocations(&object.code);
    assert!(object.validate().is_err());
}
