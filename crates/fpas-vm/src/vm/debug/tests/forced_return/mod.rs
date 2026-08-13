//! Hand-built executables for debugger forced return.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, NO_REGISTER, Opcode, Register, ReturnConvention, SourceId, SourceMap,
    SourceRun, StringId, StringTable, VerifiedExecutable,
};

pub(super) use super::*;

pub(super) fn function_return_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::Integer)
}

pub(super) fn procedure_return_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::Unit)
}

pub(super) fn array_return_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::Array)
}

pub(super) fn metadata_less_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::MissingMetadata)
}

pub(super) fn dynamic_result_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::Dynamic)
}

pub(super) fn function_result_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::Function)
}

pub(super) fn task_result_executable() -> VerifiedExecutable {
    nested_executable(ReturnKind::Task)
}

pub(super) fn panic_executable() -> VerifiedExecutable {
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("constant"),
            abc(Opcode::Panic, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![routine(
            0,
            0,
            3,
            0,
            1,
            ReturnConvention::Unit,
            debug_info(0, &[(0, 1), (1, 2)]),
        )],
        vec![Constant::String(StringId::new(4))],
        vec![(0, 1)],
        vec![DebugType::Unit],
    )
}

pub(super) fn spawn_then_call_executable() -> VerifiedExecutable {
    let mut root = routine(
        0,
        0,
        5,
        0,
        4,
        ReturnConvention::Unit,
        FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: vec![binding(5, 0, 0, DebugBindingKind::Local, true)],
            sequence_points: vec![point(0, 1), point(3, 2)],
            result_type: Some(DebugTypeId::new(1)),
        },
    );
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 3, 2).expect("task function"),
            abc(Opcode::SpawnDetachedTask, 3, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 2, 0).expect("41"),
            Instruction::abc(Opcode::CallDirect, 0, 1, 2, 1).expect("compute"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("offset"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 3).expect("sleep duration"),
            Instruction::abc(
                Opcode::Intrinsic,
                NO_REGISTER,
                u16::from(fpas_bytecode::Intrinsic::Time(
                    fpas_bytecode::TimeIntrinsic::Sleep,
                )),
                0,
                1,
            )
            .expect("sleep"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            root,
            compute_function(ReturnKind::Integer, 5),
            routine(
                3,
                8,
                11,
                0,
                1,
                ReturnConvention::Unit,
                debug_info(1, &[(8, 20)]),
            ),
        ],
        vec![
            Constant::Integer(41),
            Constant::Integer(1),
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
            Constant::Integer(30),
        ],
        vec![(0, 1), (3, 2), (5, 10), (8, 20)],
        vec![DebugType::Integer, DebugType::Unit],
    )
}

#[derive(Clone, Copy)]
enum ReturnKind {
    Integer,
    Unit,
    Array,
    MissingMetadata,
    Dynamic,
    Function,
    Task,
}

fn nested_executable(kind: ReturnKind) -> VerifiedExecutable {
    let (callee, constants, debug_types, call) = match kind {
        ReturnKind::Unit => (
            announce_function(),
            Vec::new(),
            vec![DebugType::Unit, DebugType::Integer],
            Instruction::abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 0).expect("announce"),
        ),
        ReturnKind::Array => (
            collect_function(),
            vec![Constant::Integer(1), Constant::Integer(2)],
            vec![DebugType::Integer, DebugType::Array(DebugTypeId::new(0))],
            Instruction::abc(Opcode::CallDirect, 0, 1, 0, 0).expect("collect"),
        ),
        _ => (
            compute_function(kind, 4),
            vec![Constant::Integer(41), Constant::Integer(1)],
            debug_types_for(kind),
            Instruction::abc(Opcode::CallDirect, 0, 1, 2, 1).expect("compute"),
        ),
    };
    let root_code = match kind {
        ReturnKind::Unit => vec![
            call,
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        ReturnKind::Array => vec![
            call,
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("1"),
            abc(Opcode::MakeArray, 1, 0, 1),
            abc(Opcode::Return, 1, 0, 0),
        ],
        _ => vec![
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("marker"),
            Instruction::abx(Opcode::LoadConstant, 2, 0).expect("41"),
            call,
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("offset"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
        ],
    };
    let callee_start = u32::try_from(match kind {
        ReturnKind::Unit | ReturnKind::Array => 2,
        _ => 4,
    })
    .expect("callee start");
    let root_end = callee_start;
    let root_debug = match kind {
        ReturnKind::Unit => debug_info(0, &[(0, 1), (1, 2)]),
        ReturnKind::Array => FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: vec![binding(5, 0, 1, DebugBindingKind::Local, true)],
            sequence_points: vec![point(0, 1), point(1, 2)],
            result_type: Some(DebugTypeId::new(0)),
        },
        _ => FunctionDebugInfo {
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
        },
    };
    executable(
        root_code,
        vec![
            routine(0, 0, root_end, 0, 4, ReturnConvention::Unit, root_debug),
            callee,
        ],
        constants,
        vec![(0, 1), (callee_start, 10)],
        debug_types,
    )
}

fn compute_function(kind: ReturnKind, start: u32) -> FunctionInfo {
    let result_type = match kind {
        ReturnKind::Integer => Some(DebugTypeId::new(0)),
        ReturnKind::MissingMetadata => None,
        ReturnKind::Dynamic => Some(DebugTypeId::new(2)),
        ReturnKind::Function => Some(DebugTypeId::new(3)),
        ReturnKind::Task => Some(DebugTypeId::new(4)),
        ReturnKind::Unit | ReturnKind::Array => Some(DebugTypeId::new(0)),
    };
    FunctionInfo {
        name: StringId::new(1),
        code: CodeRange::new(
            InstructionAddress::new(start),
            InstructionAddress::new(start + 3),
        ),
        arity: 1,
        capture_count: 0,
        register_count: 3,
        return_convention: ReturnConvention::Value,
        flags: FunctionFlags::default(),
        debug: FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: vec![
                binding(7, 0, 0, DebugBindingKind::Parameter, false),
                binding(6, 1, 0, DebugBindingKind::Local, true),
            ],
            sequence_points: vec![point(start, 10), point(start + 1, 11)],
            result_type,
        },
    }
}

fn announce_function() -> FunctionInfo {
    FunctionInfo {
        name: StringId::new(2),
        code: CodeRange::new(InstructionAddress::new(2), InstructionAddress::new(4)),
        arity: 0,
        capture_count: 0,
        register_count: 1,
        return_convention: ReturnConvention::Unit,
        flags: FunctionFlags::default(),
        debug: FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: Vec::new(),
            sequence_points: vec![point(2, 10)],
            result_type: Some(DebugTypeId::new(0)),
        },
    }
}

fn collect_function() -> FunctionInfo {
    FunctionInfo {
        name: StringId::new(1),
        code: CodeRange::new(InstructionAddress::new(2), InstructionAddress::new(5)),
        arity: 0,
        capture_count: 0,
        register_count: 2,
        return_convention: ReturnConvention::Value,
        flags: FunctionFlags::default(),
        debug: FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: Vec::new(),
            sequence_points: vec![point(2, 10)],
            result_type: Some(DebugTypeId::new(1)),
        },
    }
}

fn debug_types_for(kind: ReturnKind) -> Vec<DebugType> {
    match kind {
        ReturnKind::Dynamic => vec![DebugType::Integer, DebugType::Unit, DebugType::Dynamic],
        ReturnKind::Function => vec![
            DebugType::Integer,
            DebugType::Unit,
            DebugType::Dynamic,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ],
        ReturnKind::Task => vec![
            DebugType::Integer,
            DebugType::Unit,
            DebugType::Dynamic,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
            DebugType::Task(DebugTypeId::new(0)),
        ],
        _ => vec![DebugType::Integer, DebugType::Unit],
    }
}

fn binding(
    name: u32,
    register: u16,
    ty: u32,
    kind: DebugBindingKind,
    mutable: bool,
) -> DebugBinding {
    DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(8),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind,
        mutable,
        scope: 0,
        declaration: Some(fpas_bytecode::DebugSourceLocation {
            source: SourceId::new(0),
            line: 10,
            column: 3,
        }),
        hidden: false,
        cell_backed: false,
    }
}

fn debug_info(result: u32, points: &[(u32, u32)]) -> FunctionDebugInfo {
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
        result_type: Some(DebugTypeId::new(result)),
    }
}

fn routine(
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

fn executable(
    code: Vec<Instruction>,
    functions: Vec<FunctionInfo>,
    constants: Vec<Constant>,
    runs: Vec<(u32, u32)>,
    debug_types: Vec<DebugType>,
) -> VerifiedExecutable {
    Executable {
        code,
        functions,
        constants,
        strings: StringTable::new(
            [
                "root",
                "compute",
                "announce",
                "task",
                "test.fpas",
                "Answer",
                "Offset",
                "Value",
                "Integer",
                "Marker",
                "Items",
                "boom",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        ),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types,
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
    .expect("forced-return fixture executable")
}

pub(super) fn name(name: &str) -> DebugExpression {
    DebugExpression::Name(name.to_string())
}

pub(super) fn int_expr(value: i64) -> DebugExpression {
    DebugExpression::Integer(value)
}

pub(super) fn stopped(result: DebugRunResult) -> super::super::DebugStop {
    let DebugRunResult::Stopped(stop) = result else {
        panic!("expected stopped debug result")
    };
    stop
}

pub(super) fn stop_in_callee(session: &mut DebugSession, name: &str) -> u64 {
    for _ in 0..32 {
        let stack = session.stack(0, 8).expect("stack");
        if stack
            .items
            .first()
            .is_some_and(|frame| frame.name == name && frame.depth == 0)
            && session.last_stop().call_depth >= 1
        {
            return stack.items[0].id;
        }
        let _ = stopped(session.step_into().expect("step into callee"));
    }
    panic!("{name} never became the active callee")
}

pub(super) fn named<'a>(
    variables: &'a [super::super::DebugVariable],
    name: &str,
) -> &'a super::super::DebugVariable {
    variables
        .iter()
        .find(|variable| variable.name == name)
        .unwrap_or_else(|| panic!("{name} should exist"))
}

pub(super) fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}

mod cases;
