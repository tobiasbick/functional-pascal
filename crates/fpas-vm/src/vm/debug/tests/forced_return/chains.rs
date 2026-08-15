//! Multi-frame call-chain executables for selected-frame forced return.

use fpas_bytecode::{
    CodeRange, Constant, DebugBindingKind, DebugScope, DebugType, DebugTypeId, Executable,
    FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress,
    NO_REGISTER, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun, StringId, StringTable,
    VerifiedExecutable,
};

use super::fixtures::*;

pub(super) fn three_level_executable() -> VerifiedExecutable {
    chain_executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 1, 0).expect("marker"),
            Instruction::abx(Opcode::LoadConstant, 2, 1).expect("arg"),
            Instruction::abc(Opcode::CallDirect, 0, 1, 2, 1).expect("branch"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 2).expect("ten"),
            abc(Opcode::AddInteger, 1, 0, 1),
            Instruction::abc(Opcode::CallDirect, 2, 2, 1, 1).expect("leaf"),
            abc(Opcode::Return, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("one"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
        ],
        vec![
            named_routine(0, 0, 4, 0, 4, ReturnConvention::Unit, root_debug()),
            named_routine(1, 4, 8, 1, 3, ReturnConvention::Value, branch_debug()),
            named_routine(2, 8, 11, 1, 3, ReturnConvention::Value, leaf_debug()),
        ],
        vec![
            Constant::Integer(7),
            Constant::Integer(1),
            Constant::Integer(10),
        ],
        vec![(0, 1), (4, 10), (8, 20)],
    )
}

pub(super) fn four_level_executable() -> VerifiedExecutable {
    chain_executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 1, 0).expect("arg"),
            Instruction::abc(Opcode::CallDirect, 0, 1, 1, 1).expect("alpha"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("ten"),
            abc(Opcode::AddInteger, 2, 0, 1),
            Instruction::abc(Opcode::CallDirect, 3, 2, 2, 1).expect("beta"),
            abc(Opcode::Return, 3, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 2).expect("hundred"),
            abc(Opcode::AddInteger, 2, 0, 1),
            Instruction::abc(Opcode::CallDirect, 3, 3, 2, 1).expect("gamma"),
            abc(Opcode::Return, 3, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 0).expect("one"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
        ],
        vec![
            named_routine(
                0,
                0,
                3,
                0,
                2,
                ReturnConvention::Unit,
                debug_info(1, &[(0, 1), (1, 2)]),
            ),
            named_routine(
                12,
                3,
                7,
                1,
                4,
                ReturnConvention::Value,
                debug_info(0, &[(3, 10)]),
            ),
            named_routine(
                13,
                7,
                11,
                1,
                4,
                ReturnConvention::Value,
                debug_info(0, &[(7, 20)]),
            ),
            named_routine(
                14,
                11,
                14,
                1,
                3,
                ReturnConvention::Value,
                debug_info(0, &[(11, 30)]),
            ),
        ],
        vec![
            Constant::Integer(1),
            Constant::Integer(10),
            Constant::Integer(100),
        ],
        vec![(0, 1), (3, 10), (7, 20), (11, 30)],
    )
}

pub(super) fn mixed_procedure_over_function_executable() -> VerifiedExecutable {
    chain_executable(
        vec![
            Instruction::abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 0).expect("middle"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("arg"),
            Instruction::abc(Opcode::CallDirect, 1, 2, 0, 1).expect("inner"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, 0, 0, 0),
        ],
        vec![
            named_routine(
                0,
                0,
                2,
                0,
                1,
                ReturnConvention::Unit,
                debug_info(1, &[(0, 1)]),
            ),
            named_routine(
                16,
                2,
                5,
                0,
                2,
                ReturnConvention::Unit,
                debug_info(1, &[(2, 10)]),
            ),
            named_routine(
                17,
                5,
                6,
                1,
                1,
                ReturnConvention::Value,
                debug_info(0, &[(5, 20)]),
            ),
        ],
        vec![Constant::Integer(1)],
        vec![(0, 1), (2, 10), (5, 20)],
    )
}

pub(super) fn mixed_function_over_procedure_executable() -> VerifiedExecutable {
    chain_executable(
        vec![
            Instruction::abc(Opcode::CallDirect, 0, 1, 0, 0).expect("middle"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abc(Opcode::CallDirect, NO_REGISTER, 2, 0, 0).expect("inner"),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("unused"),
            abc(Opcode::Return, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            named_routine(
                0,
                0,
                2,
                0,
                1,
                ReturnConvention::Unit,
                FunctionDebugInfo {
                    scopes: vec![DebugScope {
                        id: 0,
                        parent: None,
                    }],
                    bindings: vec![binding(5, 0, 0, DebugBindingKind::Local, true)],
                    sequence_points: vec![point(0, 1)],
                    result_type: Some(DebugTypeId::new(1)),
                    ..Default::default()
                },
            ),
            named_routine(
                16,
                2,
                5,
                0,
                1,
                ReturnConvention::Value,
                debug_info(0, &[(2, 10)]),
            ),
            named_routine(
                17,
                5,
                6,
                0,
                1,
                ReturnConvention::Unit,
                debug_info(1, &[(5, 20)]),
            ),
        ],
        vec![Constant::Integer(0)],
        vec![(0, 1), (2, 10), (5, 20)],
    )
}

fn root_debug() -> FunctionDebugInfo {
    FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            binding(5, 0, 0, DebugBindingKind::Local, true),
            binding(9, 1, 0, DebugBindingKind::Local, true),
        ],
        sequence_points: vec![point(0, 1), point(2, 2)],
        result_type: Some(DebugTypeId::new(1)),
        ..Default::default()
    }
}

fn branch_debug() -> FunctionDebugInfo {
    FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            binding(7, 0, 0, DebugBindingKind::Parameter, false),
            binding(15, 1, 0, DebugBindingKind::Local, true),
            binding(18, 2, 0, DebugBindingKind::Local, true),
        ],
        sequence_points: vec![point(4, 10), point(6, 11)],
        result_type: Some(DebugTypeId::new(0)),
        ..Default::default()
    }
}

fn leaf_debug() -> FunctionDebugInfo {
    FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            binding(7, 0, 0, DebugBindingKind::Parameter, false),
            binding(19, 1, 0, DebugBindingKind::Local, true),
        ],
        sequence_points: vec![point(8, 20)],
        result_type: Some(DebugTypeId::new(0)),
        ..Default::default()
    }
}

fn named_routine(
    name: u32,
    start: u32,
    end: u32,
    arity: u8,
    registers: u16,
    convention: ReturnConvention,
    debug: FunctionDebugInfo,
) -> FunctionInfo {
    FunctionInfo {
        name: StringId::new(name),
        code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
        arity,
        capture_count: 0,
        register_count: registers,
        return_convention: convention,
        flags: FunctionFlags::default(),
        debug,
    }
}

fn chain_executable(
    code: Vec<Instruction>,
    functions: Vec<FunctionInfo>,
    constants: Vec<Constant>,
    runs: Vec<(u32, u32)>,
) -> VerifiedExecutable {
    Executable {
        code,
        functions,
        constants,
        strings: StringTable::new(
            [
                "root",
                "branch",
                "leaf",
                "task",
                "test.fpas",
                "Answer",
                "Offset",
                "Value",
                "Integer",
                "Marker",
                "Items",
                "boom",
                "alpha",
                "beta",
                "gamma",
                "Local",
                "middle",
                "inner",
                "Nested",
                "Inner",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        ),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![DebugType::Integer, DebugType::Unit],
        source_map: SourceMap {
            sources: vec![StringId::new(4)],
            runs: runs
                .into_iter()
                .map(|(instruction, line)| SourceRun {
                    instruction_start: InstructionAddress::new(instruction),
                    source: SourceId::new(0),
                    line,
                    column: 3,
                })
                .collect(),
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("selected-frame chain executable")
}
