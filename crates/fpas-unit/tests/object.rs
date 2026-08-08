#![allow(
    clippy::expect_used,
    reason = "object fixtures use expect for compact round-trip assertions"
)]

use std::collections::BTreeMap;

use fpas_bytecode::{Chunk, Op, SourceLocation, Value};
use fpas_unit::object::{
    ChunkConstant as ObjectConstant, ChunkFunction as ObjectFunction,
    ChunkLocation as ObjectLocation, ChunkObject as RelocatableObject,
    ChunkObjectError as ObjectError, collect_chunk_relocations as collect_relocations,
    decode_chunk_object as decode_object, encode_chunk_object as encode_object,
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

#[test]
fn object_from_chunk_preserves_constants_locations_and_functions() {
    let mut chunk = Chunk::new();
    let name = chunk
        .add_constant(Value::Str(("demo.unit.run".to_string()).into()))
        .expect("constant");
    chunk.emit(Op::Constant(name), SourceLocation::new_with_source(4, 2, 7));
    chunk.emit(Op::Halt, SourceLocation::new_with_source(5, 1, 7));
    chunk.insert_function("demo.unit.run", 0, 2);

    let object = RelocatableObject::from_chunk("demo.unit", &chunk, Vec::new(), Vec::new())
        .expect("object conversion");

    assert_eq!(
        object.constants,
        vec![ObjectConstant::String("demo.unit.run".to_string())]
    );
    assert_eq!(object.locations[0].source_id, 7);
    assert_eq!(object.functions["demo.unit.run"].arity, 2);
    assert_eq!(object.relocations, collect_relocations(chunk.code()));
}

#[test]
fn validation_rejects_instruction_stream_without_final_halt() {
    let mut object = object_with_all_relocation_shapes();
    object.code.pop();
    object.locations.pop();
    object.relocations = collect_relocations(&object.code);

    assert_eq!(object.validate(), Err(ObjectError::MissingHalt));
}

#[test]
fn validation_rejects_multiple_trailing_halts() {
    let mut object = object_with_all_relocation_shapes();
    object.code.insert(object.code.len() - 1, Op::Halt);
    object.locations.push(ObjectLocation {
        line: 1,
        column: 1,
        source_id: 2,
    });
    object.relocations = collect_relocations(&object.code);

    assert_eq!(
        object.validate(),
        Err(ObjectError::InternalHalt {
            instruction: object.code.len() - 2,
        })
    );
}

#[test]
fn validation_rejects_internal_halt_followed_by_code() {
    let mut object = object_with_all_relocation_shapes();
    object.code.insert(1, Op::Halt);
    object.locations.push(ObjectLocation {
        line: 1,
        column: 1,
        source_id: 2,
    });
    object.relocations = collect_relocations(&object.code);

    assert_eq!(
        object.validate(),
        Err(ObjectError::InternalHalt { instruction: 1 })
    );
}

#[test]
fn validation_rejects_mismatched_location_count() {
    let mut object = object_with_all_relocation_shapes();
    object.locations.pop();

    assert_eq!(
        object.validate(),
        Err(ObjectError::LocationCount {
            code: object.code.len(),
            locations: object.code.len() - 1,
        })
    );
}

#[test]
fn validation_rejects_function_offset_outside_instruction_stream() {
    let mut object = object_with_all_relocation_shapes();
    object.functions.insert(
        "demo.unit.invalid".to_string(),
        ObjectFunction {
            code_start: object.code.len() as u32,
            arity: 0,
        },
    );

    assert!(matches!(
        object.validate(),
        Err(ObjectError::FunctionOffset { name, .. }) if name == "demo.unit.invalid"
    ));
}

#[test]
fn validation_rejects_jump_target_outside_instruction_stream() {
    let mut object = object_with_all_relocation_shapes();
    object.code[11] = Op::Jump((object.code.len() + 1) as u32);
    object.relocations = collect_relocations(&object.code);

    assert!(matches!(
        object.validate(),
        Err(ObjectError::CodeTarget {
            instruction: 11,
            target,
            code,
        }) if target == code as u32 + 1
    ));
}

#[test]
fn object_from_chunk_preserves_every_persistent_constant_kind() {
    let mut chunk = Chunk::new();
    for value in [
        Value::Integer(-7),
        Value::Real(1.5),
        Value::Boolean(true),
        Value::Str(("name".to_string()).into()),
        Value::Unit,
        Value::function("demo.run".to_string(), Vec::new(), true),
    ] {
        chunk.add_constant(value).expect("constant");
    }
    chunk.emit(Op::Halt, SourceLocation::new(1, 1));

    let object = RelocatableObject::from_chunk("demo", &chunk, Vec::new(), Vec::new())
        .expect("object conversion");

    assert_eq!(
        object.constants,
        vec![
            ObjectConstant::Integer(-7),
            ObjectConstant::Real(1.5_f64.to_bits()),
            ObjectConstant::Boolean(true),
            ObjectConstant::String("name".to_string()),
            ObjectConstant::Unit,
            ObjectConstant::Function {
                name: "demo.run".to_string(),
                task_bound: true,
            },
        ]
    );
}

#[test]
fn object_from_chunk_rejects_captured_function_constants() {
    let mut chunk = Chunk::new();
    chunk
        .add_constant(Value::function(
            "demo.closure".to_string(),
            vec![Value::Integer(1)],
            false,
        ))
        .expect("constant");
    chunk.emit(Op::Halt, SourceLocation::new(1, 1));

    assert_eq!(
        RelocatableObject::from_chunk("demo", &chunk, Vec::new(), Vec::new()),
        Err(ObjectError::UnsupportedConstant("function".to_string()))
    );
}
