//! Hand-built executables for debugger forced return.

use fpas_bytecode::{
    CodeRange, Constant, DebugBindingKind, DebugScope, DebugType, DebugTypeId, FunctionDebugInfo,
    FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode,
    ReturnConvention, StringId, VerifiedExecutable,
};

pub(super) use super::support::*;

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

pub(super) fn callee_panic_executable() -> VerifiedExecutable {
    executable(
        vec![
            Instruction::abc(Opcode::CallDirect, 0, 1, 0, 0).expect("failing call"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("panic message"),
            abc(Opcode::Panic, 0, 0, 0),
            abc(Opcode::Return, 1, 0, 0),
        ],
        vec![
            routine(
                0,
                0,
                2,
                0,
                1,
                ReturnConvention::Unit,
                debug_info(1, &[(0, 1)]),
            ),
            routine(
                1,
                2,
                5,
                0,
                2,
                ReturnConvention::Value,
                debug_info(0, &[(2, 10)]),
            ),
        ],
        vec![Constant::String(StringId::new(11))],
        vec![(0, 1), (2, 10)],
        vec![DebugType::Integer, DebugType::Unit],
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
            ..Default::default()
        },
    );
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 3, 2).expect("task function"),
            Instruction::abc(Opcode::SpawnTask, 0, 3, 0, 0).expect("retained task"),
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

pub(super) fn spawn_failing_task_executable() -> VerifiedExecutable {
    let mut root = routine(
        0,
        0,
        5,
        0,
        4,
        ReturnConvention::Unit,
        debug_info(1, &[(0, 1), (3, 2)]),
    );
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 3, 0).expect("task function"),
            Instruction::abc(Opcode::SpawnTask, 0, 3, 0, 0).expect("retained task"),
            Instruction::abx(Opcode::LoadConstant, 2, 1).expect("41"),
            Instruction::abc(Opcode::CallDirect, 1, 2, 2, 1).expect("compute"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 3).expect("panic message"),
            abc(Opcode::Panic, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 2).expect("offset"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
        ],
        vec![
            root,
            routine(
                3,
                5,
                8,
                0,
                1,
                ReturnConvention::Unit,
                debug_info(1, &[(5, 20)]),
            ),
            routine(
                1,
                8,
                11,
                1,
                3,
                ReturnConvention::Value,
                debug_info(0, &[(8, 10)]),
            ),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Integer(41),
            Constant::Integer(1),
            Constant::String(StringId::new(11)),
        ],
        vec![(0, 1), (3, 2), (5, 20), (8, 10)],
        vec![DebugType::Integer, DebugType::Unit],
    )
}

pub(super) fn spawn_value_task_executable() -> VerifiedExecutable {
    let mut root = routine(
        0,
        0,
        5,
        0,
        4,
        ReturnConvention::Unit,
        debug_info(1, &[(0, 1), (3, 2)]),
    );
    root.flags.uses_spawn_tasks = true;
    executable(
        vec![
            Instruction::abx(Opcode::LoadConstant, 3, 0).expect("task function"),
            Instruction::abc(Opcode::SpawnTask, 0, 3, 0, 0).expect("retained task"),
            Instruction::abx(Opcode::LoadConstant, 2, 1).expect("41"),
            Instruction::abc(Opcode::CallDirect, 1, 2, 2, 1).expect("compute"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 0, 3).expect("task result"),
            abc(Opcode::Return, 0, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 2).expect("offset"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
        ],
        vec![
            root,
            routine(
                3,
                5,
                7,
                0,
                1,
                ReturnConvention::Value,
                debug_info(0, &[(5, 20)]),
            ),
            routine(
                1,
                7,
                10,
                1,
                3,
                ReturnConvention::Value,
                debug_info(0, &[(7, 10)]),
            ),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Integer(41),
            Constant::Integer(1),
            Constant::Integer(7),
        ],
        vec![(0, 1), (3, 2), (5, 20), (7, 10)],
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
