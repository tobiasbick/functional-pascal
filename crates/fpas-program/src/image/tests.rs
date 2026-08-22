#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "program image tests use verified fixture construction"
)]

use fpas_bytecode::{
    CodeRange, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable,
};

use super::*;
use crate::{Digest, ProgramIdentity};

fn identity() -> ProgramIdentity {
    ProgramIdentity {
        compiler_version: "test".to_string(),
        bytecode_version: fpas_bytecode::BYTECODE_VERSION,
        source_hash: Digest::of(b"source"),
        options_hash: Digest::of(b"options"),
        units: Vec::new(),
    }
}

fn executable() -> VerifiedExecutable {
    Executable {
        code: vec![Instruction::abc(Opcode::Return, u16::MAX, 0, 0, 0).unwrap()],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        }],
        constants: Vec::new(),
        strings: StringTable::new(vec!["main".to_string(), "<memory>".to_string()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![fpas_bytecode::DebugType::Dynamic],
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 1,
            }],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .unwrap()
}

#[test]
fn constructor_installs_portable_source_paths() {
    let image = ProgramImage::new(
        identity(),
        vec!["src/main.fpas".to_string()],
        vec![Digest::of(b"source")],
        executable(),
    )
    .expect("portable image");
    let source = image.executable().executable().source_map.sources[0];

    assert_eq!(
        image.executable().executable().strings.get(source),
        Some("src/main.fpas")
    );
}

#[test]
fn constructor_rejects_windows_absolute_source_path_on_every_host() {
    let error = ProgramImage::new(
        identity(),
        vec!["C:\\src\\main.fpas".to_string()],
        vec![Digest::of(b"source")],
        executable(),
    )
    .err();

    assert_eq!(
        error,
        Some(ImageError::AbsoluteSourcePath(
            "C:\\src\\main.fpas".to_string()
        ))
    );
}

#[test]
fn constructor_rejects_wrong_source_path_count() {
    let error = ProgramImage::new(identity(), Vec::new(), Vec::new(), executable()).err();

    assert_eq!(
        error,
        Some(ImageError::SourcePathCount {
            paths: 0,
            sources: 1
        })
    );
}

#[test]
fn constructor_rejects_wrong_source_hash_count() {
    let error = ProgramImage::new(
        identity(),
        vec!["src/main.fpas".to_string()],
        Vec::new(),
        executable(),
    )
    .err();

    assert_eq!(
        error,
        Some(ImageError::SourceHashCount {
            hashes: 0,
            sources: 1,
        })
    );
}
