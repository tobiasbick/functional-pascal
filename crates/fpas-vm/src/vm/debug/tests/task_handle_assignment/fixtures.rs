//! Hand-built executables for debugger task-handle assignment.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, GlobalInfo,
    Instruction, InstructionAddress, Intrinsic, NO_REGISTER, Opcode, RecordField, RecordLayout,
    Register, ReturnConvention, SourceId, SourceMap, SourceRun, StringId, StringTable,
    TaskIntrinsic, VerifiedExecutable,
};

use super::super::*;

pub(super) fn assignment_executable() -> VerifiedExecutable {
    typed_executable(false)
}

pub(super) fn consumed_executable() -> VerifiedExecutable {
    typed_executable(true)
}

fn push(code: &mut Vec<Instruction>, instruction: Instruction) -> u32 {
    let index = u32::try_from(code.len()).expect("instruction index");
    code.push(instruction);
    index
}

fn typed_executable(consume_pending: bool) -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "helper",
            "seven",
            "nine",
            "truth",
            "test.fpas",
            "Current",
            "Pending",
            "Frozen",
            "Number",
            "Loose",
            "Slot",
            "Box",
            "Items",
            "Scores",
            "Optional",
            "Missing",
            "Outcome",
            "CellSlot",
            "Shared",
            "Hidden",
            "Wrong",
            "StopMarker",
            "Waited",
            "Arg",
            "G",
            "Job",
            "Holder",
            "Integer",
            "Dynamic",
            "a",
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
    let local = |name, register, ty, mutable, kind, hidden| DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(if ty == 7 { 29 } else { 28 }),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind,
        mutable,
        scope: 0,
        declaration: Some(location(1)),
        hidden,
        cell_backed: kind == DebugBindingKind::Capture,
    };
    let mut code = Vec::new();
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 20, 0).expect("seven"),
    );
    push(&mut code, abc_aux(Opcode::SpawnTask, 1, 20, 0, 0));
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 21, 1).expect("nine"),
    );
    push(&mut code, abc_aux(Opcode::SpawnTask, 0, 21, 0, 0));
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 20, 0).expect("frozen seven"),
    );
    push(&mut code, abc_aux(Opcode::SpawnTask, 2, 20, 0, 0));
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 21, 2).expect("truth"),
    );
    push(&mut code, abc_aux(Opcode::SpawnTask, 15, 21, 0, 0));
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 3, 3).expect("Number"),
    );
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 4, 3).expect("Loose"),
    );
    push(&mut code, abc(Opcode::Move, 22, 1, 0));
    push(&mut code, abc(Opcode::MakeRecord, 6, 0, 22));
    push(&mut code, abc(Opcode::MakeArray, 7, 22, 1));
    push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 23, 4).expect("key"),
    );
    push(&mut code, abc(Opcode::Move, 24, 1, 0));
    push(&mut code, abc(Opcode::MakeDictionary, 8, 23, 1));
    push(&mut code, abc(Opcode::MakeSome, 9, 1, 0));
    push(&mut code, abc(Opcode::MakeNone, 10, 0, 0));
    push(&mut code, abc(Opcode::MakeOk, 11, 1, 0));
    push(&mut code, abc(Opcode::MakeCell, 12, 1, 0));
    push(&mut code, abc(Opcode::Move, 13, 1, 0));
    push(&mut code, abc(Opcode::Move, 14, 1, 0));
    if consume_pending {
        push(
            &mut code,
            Instruction::abc(
                Opcode::Intrinsic,
                17,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                1,
                1,
            )
            .expect("consume Pending"),
        );
    }
    let stop_at = push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 16, 3).expect("StopMarker"),
    );
    let call_at = push(&mut code, abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 0, 1));
    let wait_at = push(
        &mut code,
        Instruction::abc(
            Opcode::Intrinsic,
            17,
            u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
            0,
            1,
        )
        .expect("wait Current"),
    );
    push(&mut code, abc(Opcode::Return, NO_REGISTER, 0, 0));
    let root_end = u32::try_from(code.len()).expect("root end");
    let helper_start = push(&mut code, abc(Opcode::Return, NO_REGISTER, 0, 0));
    let seven_start = push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 0, 5).expect("seven value"),
    );
    push(&mut code, abc(Opcode::Return, 0, 0, 0));
    let nine_start = push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 0, 6).expect("nine value"),
    );
    push(&mut code, abc(Opcode::Return, 0, 0, 0));
    let truth_start = push(
        &mut code,
        Instruction::abx(Opcode::LoadConstant, 0, 7).expect("true"),
    );
    push(&mut code, abc(Opcode::Return, 0, 0, 0));
    let truth_end = u32::try_from(code.len()).expect("truth end");
    let root_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            local(6, 0, 4, true, DebugBindingKind::Local, false),
            local(7, 1, 4, false, DebugBindingKind::Local, false),
            local(8, 2, 4, false, DebugBindingKind::Local, false),
            local(9, 3, 0, true, DebugBindingKind::Local, false),
            local(10, 4, 7, true, DebugBindingKind::Local, false),
            local(11, 5, 4, true, DebugBindingKind::Local, false),
            DebugBinding {
                type_name: StringId::new(27),
                ty: DebugTypeId::new(10),
                ..local(12, 6, 10, true, DebugBindingKind::Local, false)
            },
            local(13, 7, 8, true, DebugBindingKind::Local, false),
            local(14, 8, 9, true, DebugBindingKind::Local, false),
            local(15, 9, 11, true, DebugBindingKind::Local, false),
            local(16, 10, 11, true, DebugBindingKind::Local, false),
            local(17, 11, 12, true, DebugBindingKind::Local, false),
            local(18, 12, 4, true, DebugBindingKind::Capture, false),
            local(19, 13, 4, true, DebugBindingKind::Local, false),
            local(20, 14, 4, true, DebugBindingKind::Local, true),
            local(21, 15, 5, true, DebugBindingKind::Local, false),
            local(22, 16, 0, true, DebugBindingKind::Local, false),
            local(23, 17, 0, true, DebugBindingKind::Local, false),
        ],
        sequence_points: vec![
            point(0, 1),
            point(stop_at, 2),
            point(call_at, 10),
            point(wait_at, 3),
        ],
        ..Default::default()
    };
    let helper_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![local(24, 0, 4, true, DebugBindingKind::Parameter, false)],
        sequence_points: vec![point(helper_start, 10)],
        result_type: Some(DebugTypeId::new(1)),
        ..Default::default()
    };
    let scalar = |result_type| FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        result_type: Some(DebugTypeId::new(result_type)),
        ..Default::default()
    };
    let routine = |name, start, end, arity, registers, convention, flags, debug| FunctionInfo {
        name: StringId::new(name),
        code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
        arity,
        capture_count: 0,
        register_count: registers,
        return_convention: convention,
        flags,
        debug,
    };
    Executable {
        code,
        functions: vec![
            routine(
                0,
                0,
                root_end,
                0,
                26,
                ReturnConvention::Unit,
                FunctionFlags {
                    uses_spawn_tasks: true,
                },
                root_debug,
            ),
            routine(
                1,
                helper_start,
                seven_start,
                1,
                1,
                ReturnConvention::Unit,
                FunctionFlags::default(),
                helper_debug,
            ),
            routine(
                2,
                seven_start,
                nine_start,
                0,
                1,
                ReturnConvention::Value,
                FunctionFlags::default(),
                scalar(0),
            ),
            routine(
                3,
                nine_start,
                truth_start,
                0,
                1,
                ReturnConvention::Value,
                FunctionFlags::default(),
                scalar(0),
            ),
            routine(
                4,
                truth_start,
                truth_end,
                0,
                1,
                ReturnConvention::Value,
                FunctionFlags::default(),
                scalar(2),
            ),
        ],
        constants: vec![
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(3),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(4),
                task_bound: false,
            },
            Constant::Integer(0),
            Constant::String(StringId::new(30)),
            Constant::Integer(7),
            Constant::Integer(9),
            Constant::Boolean(true),
        ],
        strings,
        globals: vec![GlobalInfo {
            name: StringId::new(25),
            ty: DebugTypeId::new(4),
            mutable: true,
        }],
        records: vec![RecordLayout {
            name: StringId::new(27),
            fields: vec![RecordField {
                name: StringId::new(26),
                ty: DebugTypeId::new(4),
            }],
            properties: Vec::new(),
            methods: Vec::new(),
        }],
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![
            DebugType::Integer,
            DebugType::Unit,
            DebugType::Boolean,
            DebugType::String,
            DebugType::Task(DebugTypeId::new(0)),
            DebugType::Task(DebugTypeId::new(2)),
            DebugType::Task(DebugTypeId::new(3)),
            DebugType::Dynamic,
            DebugType::Array(DebugTypeId::new(4)),
            DebugType::Dictionary {
                key: DebugTypeId::new(3),
                value: DebugTypeId::new(4),
            },
            DebugType::Record(fpas_bytecode::RecordTypeId::new(0)),
            DebugType::Option(DebugTypeId::new(4)),
            DebugType::Result {
                ok: DebugTypeId::new(4),
                error: DebugTypeId::new(3),
            },
        ],
        source_map: SourceMap {
            sources: vec![StringId::new(5)],
            runs: [
                0,
                stop_at,
                call_at,
                wait_at,
                helper_start,
                seven_start,
                nine_start,
                truth_start,
            ]
            .into_iter()
            .enumerate()
            .map(|(index, instruction)| SourceRun {
                instruction_start: InstructionAddress::new(instruction),
                source: SourceId::new(0),
                line: u32::try_from(index.saturating_add(1)).expect("line"),
                column: 3,
            })
            .collect(),
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("task-handle assignment executable")
}
