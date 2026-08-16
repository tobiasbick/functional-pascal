//! Hand-built child-task executable for selected-task assignment coverage.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Intrinsic, NO_REGISTER, Opcode, Register, ReturnConvention, SourceId,
    SourceMap, SourceRun, StringId, StringTable, TaskIntrinsic, VerifiedExecutable,
};

use super::super::*;

pub(super) fn child_task_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "work",
            "seven",
            "nine",
            "test.fpas",
            "Current",
            "Backup",
            "Pending",
            "Integer",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    );
    let location = |line| fpas_bytecode::DebugSourceLocation {
        source: SourceId::new(0),
        line,
        column: 3,
    };
    let local = |name, register, ty, mutable, line| DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(8),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind: DebugBindingKind::Local,
        mutable,
        scope: 0,
        declaration: Some(location(line)),
        hidden: false,
        cell_backed: false,
        initializer: None,
    };
    let scalar = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        result_type: Some(DebugTypeId::new(0)),
        ..Default::default()
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("work"),
            abc_aux(Opcode::SpawnTask, 1, 0, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                2,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                1,
                1,
            )
            .expect("wait work"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 2, 1).expect("seven"),
            abc_aux(Opcode::SpawnTask, 0, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 3, 2).expect("nine"),
            abc_aux(Opcode::SpawnTask, 1, 3, 0, 0),
            abc(Opcode::Move, 1, 1, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                0,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                1,
                1,
            )
            .expect("wait current"),
            abc(Opcode::Return, 0, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 3).expect("7"),
            abc(Opcode::Return, 0, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 4).expect("9"),
            abc(Opcode::Return, 0, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(4)),
                arity: 0,
                capture_count: 0,
                register_count: 3,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags {
                    uses_spawn_tasks: true,
                },
                debug: FunctionDebugInfo {
                    scopes: vec![DebugScope {
                        id: 0,
                        parent: None,
                    }],
                    bindings: vec![local(7, 1, 2, true, 1)],
                    sequence_points: vec![point(0, 1), point(2, 2)],
                    ..Default::default()
                },
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(4), InstructionAddress::new(11)),
                arity: 0,
                capture_count: 0,
                register_count: 4,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags {
                    uses_spawn_tasks: true,
                },
                debug: FunctionDebugInfo {
                    scopes: vec![DebugScope {
                        id: 0,
                        parent: None,
                    }],
                    bindings: vec![local(6, 0, 2, false, 20), local(5, 1, 2, true, 20)],
                    sequence_points: vec![point(4, 20), point(8, 21)],
                    result_type: Some(DebugTypeId::new(0)),
                    ..Default::default()
                },
            },
            FunctionInfo {
                name: StringId::new(2),
                code: CodeRange::new(InstructionAddress::new(11), InstructionAddress::new(13)),
                arity: 0,
                capture_count: 0,
                register_count: 1,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags::default(),
                debug: scalar.clone(),
            },
            FunctionInfo {
                name: StringId::new(3),
                code: CodeRange::new(InstructionAddress::new(13), InstructionAddress::new(15)),
                arity: 0,
                capture_count: 0,
                register_count: 1,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags::default(),
                debug: scalar,
            },
        ],
        constants: vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(3),
                task_bound: false,
            },
            Constant::Integer(7),
            Constant::Integer(9),
        ],
        strings,
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![
            DebugType::Integer,
            DebugType::Unit,
            DebugType::Task(DebugTypeId::new(0)),
        ],
        source_map: SourceMap {
            sources: vec![StringId::new(4)],
            runs: vec![
                SourceRun {
                    instruction_start: InstructionAddress::new(0),
                    source: SourceId::new(0),
                    line: 1,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(4),
                    source: SourceId::new(0),
                    line: 20,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(8),
                    source: SourceId::new(0),
                    line: 21,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(11),
                    source: SourceId::new(0),
                    line: 30,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(13),
                    source: SourceId::new(0),
                    line: 31,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("child task-handle assignment executable")
}
