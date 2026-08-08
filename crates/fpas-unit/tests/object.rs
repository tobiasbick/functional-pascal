//! Relocatable register-object conversion, codec, and validation tests.

#![allow(
    clippy::expect_used,
    reason = "focused object fixtures use explicit expectations"
)]

use fpas_bytecode::{
    CodeRange, Constant, EnumLayout, EnumTypeId, EnumVariant, Executable, FunctionFlags,
    FunctionId, FunctionInfo, GlobalInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode,
    RecordField, RecordLayout, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable,
};
use fpas_unit::object::{
    OBJECT_VERSION, ObjectError, RelocatableObject, RelocationKind, decode_object, encode_object,
};

fn abc(opcode: Opcode, a: u16, b: u16, c: u16) -> Instruction {
    Instruction::abc(opcode, a, b, c, 0).expect("valid ABC fixture")
}

fn candidate() -> Executable {
    let code = vec![
        Instruction::abx(Opcode::LoadConstant, 0, 0).expect("constant"),
        Instruction::abx(Opcode::StoreGlobal, 0, 0).expect("global"),
        abc(Opcode::MakeRecord, 1, 0, 0),
        abc(Opcode::LoadField, 2, 1, 0),
        abc(Opcode::MakeEnum, 3, 0, 2),
        abc(Opcode::TestVariant, 4, 3, 0),
        abc(Opcode::LoadEnumField, 5, 3, 0),
        Instruction::abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 0).expect("direct call"),
        abc(Opcode::Return, NO_REGISTER, 0, 0),
        abc(Opcode::Return, NO_REGISTER, 0, 0),
    ];
    Executable {
        code,
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(9)),
                arity: 0,
                capture_count: 0,
                register_count: 6,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(9), InstructionAddress::new(10)),
                arity: 0,
                capture_count: 0,
                register_count: 0,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
            },
        ],
        constants: vec![Constant::String(StringId::new(2))],
        strings: StringTable::new(vec![
            "demo.main".to_string(),
            "demo.helper".to_string(),
            "value".to_string(),
            "demo.global".to_string(),
            "demo.record".to_string(),
            "field".to_string(),
            "demo.enum".to_string(),
            "item".to_string(),
            "fixture.fpas".to_string(),
        ]),
        globals: vec![GlobalInfo {
            name: StringId::new(3),
            mutable: true,
        }],
        records: vec![RecordLayout {
            name: StringId::new(4),
            fields: vec![RecordField {
                name: StringId::new(5),
            }],
        }],
        enums: vec![EnumLayout {
            name: StringId::new(6),
        }],
        enum_variants: vec![EnumVariant {
            owner: EnumTypeId::new(0),
            name: StringId::new(7),
            fields: vec![StringId::new(5)],
        }],
        source_map: SourceMap {
            sources: vec![StringId::new(8)],
            runs: vec![
                SourceRun {
                    instruction_start: InstructionAddress::new(0),
                    source: SourceId::new(0),
                    line: 1,
                    column: 1,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(9),
                    source: SourceId::new(0),
                    line: 4,
                    column: 1,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
}

fn object() -> RelocatableObject {
    RelocatableObject::from_executable(
        "demo.program",
        candidate().verify().expect("verified fixture"),
    )
    .expect("relocatable object")
}

#[test]
fn conversion_covers_every_register_table_operand_and_is_deterministic() {
    let first = object();
    let second = object();
    assert_eq!(first, second);
    assert_eq!(first.version, OBJECT_VERSION);
    assert_eq!(first.functions[1].source_runs[0].instruction_start, 0);
    assert!(
        first
            .relocations
            .iter()
            .any(|relocation| matches!(relocation.kind, RelocationKind::Function(_)))
    );
    assert!(
        first
            .relocations
            .iter()
            .any(|relocation| matches!(relocation.kind, RelocationKind::EnumVariant { .. }))
    );
    let first_bytes = encode_object(&first).expect("first encoding");
    let second_bytes = encode_object(&second).expect("second encoding");
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(decode_object(&first_bytes).expect("round trip"), first);
}

#[test]
fn incompatible_object_version_is_rejected_before_linking() {
    let mut object = object();
    object.version = OBJECT_VERSION - 1;
    assert_eq!(
        object.validate(),
        Err(ObjectError::Version {
            actual: OBJECT_VERSION - 1,
            expected: OBJECT_VERSION,
        })
    );
}

#[test]
fn missing_and_wrong_relocation_coverage_is_rejected() {
    let mut missing = object();
    missing.relocations.remove(0);
    assert!(matches!(
        missing.validate(),
        Err(ObjectError::RelocationCoverage { .. })
    ));

    let mut wrong = object();
    wrong.relocations[0].kind = RelocationKind::CodeAddress(0);
    assert!(matches!(
        wrong.validate(),
        Err(ObjectError::RelocationCoverage { .. })
    ));
}

#[test]
fn every_truncated_object_prefix_is_rejected_without_panic() {
    let bytes = encode_object(&object()).expect("object encoding");
    for length in 0..bytes.len() {
        assert!(
            decode_object(&bytes[..length]).is_err(),
            "truncated prefix {length} must fail"
        );
    }
}

#[test]
fn duplicate_case_insensitive_definition_is_rejected_by_canonical_contract() {
    let mut object = object();
    let duplicate = object.definitions[0].clone();
    object.definitions.push(duplicate);
    assert!(matches!(
        object.validate(),
        Err(ObjectError::DuplicateName(_))
    ));
}
