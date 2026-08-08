#![expect(
    clippy::expect_used,
    reason = "program codec fixtures use direct construction assertions"
)]
#![allow(
    dead_code,
    reason = "each integration-test crate uses a subset of shared fixtures"
)]

use fpas_bytecode::{
    CodeRange, Constant, EnumLayout, EnumTypeId, EnumVariant, Executable, FunctionFlags,
    FunctionId, FunctionInfo, GlobalInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode,
    RecordField, RecordLayout, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable,
};
use fpas_program::{Digest, LinkedUnitIdentity, ProgramIdentity, ProgramImage};

pub fn program_image() -> ProgramImage {
    let strings = vec![
        "root",
        "test.fpas",
        "hello",
        "global",
        "record",
        "field",
        "enum",
        "variant",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let executable = Executable {
        code: vec![Instruction::abc(Opcode::Return, NO_REGISTER, 0, 0, 0).expect("return")],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
        }],
        constants: vec![
            Constant::Integer(i64::MIN),
            Constant::Real(0.0_f64.to_bits()),
            Constant::Real((-0.0_f64).to_bits()),
            Constant::Real(f64::INFINITY.to_bits()),
            Constant::Real(0x7ff8_0000_0000_0001),
            Constant::Real(0x7ff8_0000_0000_0002),
            Constant::Boolean(true),
            Constant::String(StringId::new(2)),
            Constant::Unit,
            Constant::Function {
                function: FunctionId::new(0),
                task_bound: false,
            },
        ],
        strings: StringTable::new(strings),
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
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 3,
                column: 5,
            }],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("fixture executable");
    ProgramImage::new(
        ProgramIdentity {
            compiler_version: "test".to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: Digest::of(b"source"),
            options_hash: Digest::of(b"options"),
            units: vec![LinkedUnitIdentity {
                unit_name: "Demo.Unit".to_string(),
                object_hash: Digest::of(b"unit"),
            }],
        },
        vec!["test.fpas".to_string()],
        executable,
    )
    .expect("fixture image")
}

pub fn payload_start(bytes: &[u8]) -> usize {
    let compiler_len = u32::from_le_bytes(bytes[16..20].try_into().expect("compiler length"));
    let units_offset = 20 + compiler_len as usize + 64;
    let unit_count = u32::from_le_bytes(
        bytes[units_offset..units_offset + 4]
            .try_into()
            .expect("unit count"),
    ) as usize;
    let mut offset = units_offset + 4;
    for _ in 0..unit_count {
        let name_len = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("unit name length"),
        ) as usize;
        offset += 4 + name_len + 32;
    }
    offset + 4 + 32
}

pub fn refresh_payload_digest(bytes: &mut [u8]) {
    let start = payload_start(bytes);
    let digest_offset = start - 32;
    let digest = Digest::of(&bytes[start..]);
    bytes[digest_offset..start].copy_from_slice(digest.as_bytes());
}
