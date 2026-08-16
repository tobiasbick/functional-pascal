//! Focused controlled-execution tests independent from protocol adapters.

use fpas_bytecode::{
    CodeRange, Constant, DebugScope, DebugSourceLocation, Executable, FunctionDebugInfo,
    FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress, Intrinsic,
    NO_REGISTER, Opcode, ReturnConvention, SequencePoint, SourceId, SourceMap, SourceRun, StringId,
    StringTable, TaskIntrinsic, TimeIntrinsic, VerifiedExecutable,
};

use super::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugBinaryOperation, DebugBreakpointLimits,
    DebugErrorKind, DebugEvaluationLimits, DebugExecutionLimits, DebugExpression,
    DebugInspectionLimits, DebugRunResult, DebugSession, DebugSessionState, DebugStopReason,
    DebugTaskState, FunctionBreakpoint, SourceBreakpoint,
};

fn abc(opcode: Opcode, a: u16, b: u16, c: u16) -> Instruction {
    Instruction::abc(opcode, a, b, c, 0).expect("ABC instruction")
}

fn abc_aux(opcode: Opcode, a: u16, b: u16, c: u16, auxiliary: u8) -> Instruction {
    Instruction::abc(opcode, a, b, c, auxiliary).expect("ABC instruction")
}

fn point(instruction: u32, line: u32) -> SequencePoint {
    point_at(instruction, line, 3)
}

fn point_at(instruction: u32, line: u32, column: u32) -> SequencePoint {
    SequencePoint {
        instruction: InstructionAddress::new(instruction),
        location: DebugSourceLocation {
            source: SourceId::new(0),
            line,
            column,
        },
        scope: 0,
    }
}

fn same_line_executable() -> VerifiedExecutable {
    let mut metadata = debug(&[]);
    metadata.sequence_points = vec![point_at(0, 1, 3), point_at(1, 1, 20)];
    executable(
        vec![
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![function("root", 0, 3, 1, metadata)],
        Vec::new(),
        vec![(0, 1)],
    )
}

fn loop_executable() -> VerifiedExecutable {
    executable(
        vec![
            abc(Opcode::LoadUnit, 0, 0, 0),
            Instruction::abx(Opcode::Jump, 0, 0).expect("jump"),
        ],
        vec![function("root", 0, 2, 1, debug(&[(0, 1)]))],
        Vec::new(),
        vec![(0, 1)],
    )
}

fn blocking_intrinsic_executable() -> VerifiedExecutable {
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("sleep duration"),
            Instruction::abc(
                Opcode::Intrinsic,
                NO_REGISTER,
                u16::from(Intrinsic::Time(TimeIntrinsic::Sleep)),
                0,
                1,
            )
            .expect("sleep intrinsic"),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![function("root", 0, 4, 1, debug(&[(0, 1), (2, 2)]))],
        vec![Constant::Integer(30)],
        vec![(0, 1), (2, 2)],
    )
}

fn recursive_executable() -> VerifiedExecutable {
    executable(
        vec![
            abc(Opcode::CallDirect, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![function("root", 0, 2, 0, debug(&[(0, 1)]))],
        Vec::new(),
        vec![(0, 1)],
    )
}

fn debug(points: &[(u32, u32)]) -> FunctionDebugInfo {
    FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: Vec::new(),
        sequence_points: points
            .iter()
            .map(|(instruction, line)| point(*instruction, *line))
            .collect(),
        ..Default::default()
    }
}

fn call_executable() -> VerifiedExecutable {
    let code = vec![
        abc(Opcode::LoadUnit, 0, 0, 0),
        abc(Opcode::CallDirect, NO_REGISTER, 1, 0),
        abc(Opcode::LoadUnit, 0, 0, 0),
        abc(Opcode::Return, NO_REGISTER, 0, 0),
        abc(Opcode::LoadUnit, 0, 0, 0),
        abc(Opcode::Return, NO_REGISTER, 0, 0),
    ];
    executable(
        code,
        vec![
            function("root", 0, 4, 1, debug(&[(0, 1), (1, 2), (2, 3)])),
            function("helper", 4, 6, 1, debug(&[(4, 10), (5, 11)])),
        ],
        Vec::new(),
        vec![(0, 1), (4, 10)],
    )
}

fn panic_executable() -> VerifiedExecutable {
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("constant"),
            abc(Opcode::Panic, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![function("root", 0, 3, 1, debug(&[(0, 1), (1, 2)]))],
        vec![Constant::String(StringId::new(3))],
        vec![(0, 1)],
    )
}

fn panic_without_sequence_point_executable() -> VerifiedExecutable {
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("constant"),
            abc(Opcode::Panic, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![function("root", 0, 3, 1, FunctionDebugInfo::default())],
        vec![Constant::String(StringId::new(3))],
        vec![(0, 1)],
    )
}

fn task_executable() -> VerifiedExecutable {
    let mut root = function("root", 0, 3, 1, debug(&[(0, 1)]));
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("function constant"),
            abc(Opcode::SpawnDetachedTask, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![root, function("task", 3, 4, 0, debug(&[(3, 10)]))],
        vec![Constant::Function {
            function: FunctionId::new(1),
            task_bound: false,
        }],
        vec![(0, 1), (3, 10)],
    )
}

fn same_deadline_tasks_executable() -> VerifiedExecutable {
    let mut root = function("root", 0, 6, 4, debug(&[(0, 1), (3, 2), (4, 3)]));
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("function constant"),
            abc_aux(Opcode::SpawnTask, 1, 0, 0, 0),
            abc_aux(Opcode::SpawnTask, 2, 0, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                3,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                1,
                1,
            )
            .expect("first task wait"),
            Instruction::abc(
                Opcode::Intrinsic,
                3,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                2,
                1,
            )
            .expect("second task wait"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 1).expect("sleep duration"),
            Instruction::abc(
                Opcode::Intrinsic,
                NO_REGISTER,
                u16::from(Intrinsic::Time(TimeIntrinsic::Sleep)),
                0,
                1,
            )
            .expect("task sleep"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![root, function("task", 6, 9, 1, debug(&[(6, 10), (8, 11)]))],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Integer(100),
        ],
        vec![(0, 1), (3, 2), (4, 3), (6, 10), (8, 11)],
    )
}

fn task_state_executable() -> VerifiedExecutable {
    let mut root = function("root", 0, 7, 5, debug(&[(0, 1), (4, 2), (5, 3)]));
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("sleeper function"),
            abc_aux(Opcode::SpawnTask, 2, 0, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("stopper function"),
            abc_aux(Opcode::SpawnTask, 3, 1, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                4,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                2,
                1,
            )
            .expect("sleeper wait"),
            Instruction::abc(
                Opcode::Intrinsic,
                4,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                3,
                1,
            )
            .expect("stopper wait"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 2).expect("sleep duration"),
            Instruction::abc(
                Opcode::Intrinsic,
                NO_REGISTER,
                u16::from(Intrinsic::Time(TimeIntrinsic::Sleep)),
                0,
                1,
            )
            .expect("task sleep"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            root,
            function("sleeper", 7, 10, 1, debug(&[(7, 10), (9, 11)])),
            function("stopper", 10, 14, 1, debug(&[(10, 20), (12, 21)])),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
            Constant::Integer(100),
        ],
        vec![(0, 1), (4, 2), (5, 3), (7, 10), (9, 11), (10, 20), (12, 21)],
    )
}

fn yield_precedence_executable() -> VerifiedExecutable {
    let mut root = function("root", 0, 7, 5, debug(&[(0, 1), (4, 2), (5, 3)]));
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("yielding function"),
            abc_aux(Opcode::SpawnTask, 2, 0, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("breakpoint function"),
            abc_aux(Opcode::SpawnTask, 3, 1, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                4,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                2,
                1,
            )
            .expect("yielding task wait"),
            Instruction::abc(
                Opcode::Intrinsic,
                4,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                3,
                1,
            )
            .expect("breakpoint task wait"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Yield, 0, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            root,
            function("yielding", 7, 10, 1, debug(&[(7, 10), (8, 11)])),
            function("breakpoint", 10, 12, 1, debug(&[(10, 20)])),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
        ],
        vec![(0, 1), (4, 2), (5, 3), (7, 10), (8, 11), (10, 20)],
    )
}

fn unreachable_task_executable() -> VerifiedExecutable {
    let mut spawner = function("spawner", 1, 4, 1, debug(&[(1, 10)]));
    spawner.flags.uses_spawn_tasks = true;
    executable(
        vec![
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("function constant"),
            abc(Opcode::SpawnDetachedTask, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            function("root", 0, 1, 0, debug(&[(0, 1)])),
            spawner,
            function("task", 4, 5, 0, debug(&[(4, 20)])),
        ],
        vec![Constant::Function {
            function: FunctionId::new(2),
            task_bound: false,
        }],
        vec![(0, 1), (1, 10), (4, 20)],
    )
}

fn function(
    name: &str,
    start: u32,
    end: u32,
    registers: u16,
    debug: FunctionDebugInfo,
) -> FunctionInfo {
    FunctionInfo {
        name: StringId::new(if name == "root" { 0 } else { 1 }),
        code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
        arity: 0,
        capture_count: 0,
        register_count: registers,
        return_convention: ReturnConvention::Unit,
        flags: FunctionFlags::default(),
        debug,
    }
}

fn executable(
    code: Vec<Instruction>,
    functions: Vec<FunctionInfo>,
    constants: Vec<Constant>,
    runs: Vec<(u32, u32)>,
) -> VerifiedExecutable {
    Executable {
        code,
        functions,
        constants,
        strings: StringTable::new(vec![
            "root".to_string(),
            "helper".to_string(),
            "test.fpas".to_string(),
            "boom".to_string(),
        ]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![fpas_bytecode::DebugType::Dynamic],
        source_map: SourceMap {
            sources: vec![StringId::new(2)],
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
    .expect("debug fixture executable")
}

fn stopped(result: DebugRunResult) -> super::DebugStop {
    let DebugRunResult::Stopped(stop) = result else {
        panic!("expected stopped debug result")
    };
    stop
}

mod behavior;
mod breakpoints;
mod capturing_routine_assignment;
mod cell_capturing_routine_assignment;
mod empty_storage_construction;
mod evaluation;
mod forced_return;
mod function_value_assignment;
mod inspection_fixture;
mod mutation;
mod payload_mutation;
mod task_handle_assignment;
mod uninitialized_assignment;
mod variant_construction;
mod variant_replacement;
mod variant_transition;

use inspection_fixture::inspection_executable;
