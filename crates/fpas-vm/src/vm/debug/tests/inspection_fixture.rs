//! Executable fixture with globals, shadowing, frames, and aggregates.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugSourceLocation,
    Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, GlobalInfo,
    Instruction, InstructionAddress, NO_REGISTER, Opcode, Register, ReturnConvention,
    SequencePoint, SourceId, SourceMap, SourceRun, StringId, StringTable, VerifiedExecutable,
};

use super::{abc, abc_aux, point};

pub(super) fn inspection_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "helper",
            "test.fpas",
            "boom",
            "Answer",
            "Inner",
            "Outside",
            "$hidden",
            "Integer",
            "Text",
            "G",
            "Value",
            "Items",
            "array of Integer",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    );
    let location = |line| DebugSourceLocation {
        source: SourceId::new(0),
        line,
        column: 3,
    };
    let binding = |name, register, scope, hidden| DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(8),
        ty: fpas_bytecode::DebugTypeId::new(0),
        register: Register::new(register).expect("register"),
        kind: DebugBindingKind::Local,
        mutable: true,
        scope,
        declaration: Some(location(1)),
        hidden,
        cell_backed: false,
    };
    let root_debug = FunctionDebugInfo {
        scopes: vec![
            DebugScope {
                id: 0,
                parent: None,
            },
            DebugScope {
                id: 1,
                parent: Some(0),
            },
            DebugScope {
                id: 2,
                parent: Some(0),
            },
        ],
        bindings: vec![
            binding(4, 0, 0, false),
            DebugBinding {
                type_name: StringId::new(9),
                ty: fpas_bytecode::DebugTypeId::new(1),
                ..binding(5, 1, 1, false)
            },
            binding(4, 1, 1, false),
            binding(6, 4, 2, false),
            binding(7, 4, 1, true),
            DebugBinding {
                name: StringId::new(12),
                type_name: StringId::new(13),
                ty: fpas_bytecode::DebugTypeId::new(2),
                register: Register::new(2).expect("register"),
                kind: DebugBindingKind::Local,
                mutable: true,
                scope: 1,
                declaration: Some(location(3)),
                hidden: false,
                cell_backed: false,
            },
        ],
        sequence_points: vec![
            point(0, 1),
            SequencePoint {
                instruction: InstructionAddress::new(6),
                location: location(4),
                scope: 1,
            },
        ],
        ..Default::default()
    };
    let helper_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![DebugBinding {
            name: StringId::new(11),
            type_name: StringId::new(8),
            ty: fpas_bytecode::DebugTypeId::new(0),
            register: Register::new(0).expect("register"),
            kind: DebugBindingKind::Parameter,
            mutable: false,
            scope: 0,
            declaration: Some(location(10)),
            hidden: false,
            cell_backed: false,
        }],
        sequence_points: vec![point(8, 10)],
        ..Default::default()
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("integer constant"),
            Instruction::abx(Opcode::StoreGlobal, 0, 0).expect("global store"),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("string constant"),
            Instruction::abx(Opcode::LoadConstant, 3, 2).expect("array value"),
            Instruction::abx(Opcode::LoadConstant, 4, 3).expect("array value"),
            abc(Opcode::MakeArray, 2, 3, 2),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 0, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(8)),
                arity: 0,
                capture_count: 0,
                register_count: 5,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: root_debug,
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(8), InstructionAddress::new(9)),
                arity: 1,
                capture_count: 0,
                register_count: 1,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: helper_debug,
            },
        ],
        constants: vec![
            Constant::Integer(42),
            Constant::String(StringId::new(3)),
            Constant::Integer(1),
            Constant::Integer(2),
        ],
        strings,
        globals: vec![GlobalInfo {
            name: StringId::new(10),
            ty: fpas_bytecode::DebugTypeId::new(0),
            mutable: true,
        }],
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![
            fpas_bytecode::DebugType::Integer,
            fpas_bytecode::DebugType::String,
            fpas_bytecode::DebugType::Array(fpas_bytecode::DebugTypeId::new(0)),
        ],
        source_map: SourceMap {
            sources: vec![StringId::new(2)],
            runs: vec![
                SourceRun {
                    instruction_start: InstructionAddress::new(0),
                    source: SourceId::new(0),
                    line: 1,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(8),
                    source: SourceId::new(0),
                    line: 10,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("inspection executable")
}
