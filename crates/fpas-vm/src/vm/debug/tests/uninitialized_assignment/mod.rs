//! Hand-built executables for debugger initialization of empty mutable roots.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, GlobalInfo,
    Instruction, InstructionAddress, NO_REGISTER, Opcode, RecordField, RecordLayout, RecordTypeId,
    Register, ReturnConvention, SourceId, SourceMap, SourceRun, StringId, StringTable,
    VerifiedExecutable,
};

pub(super) use super::*;

pub(super) fn assignment_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "helper",
            "test.fpas",
            "Count",
            "Frozen",
            "UnitValue",
            "Nested",
            "Items",
            "Arg",
            "Captured",
            "Integer",
            "Unit",
            "Point",
            "X",
            "Y",
            "G",
            "array of Integer",
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
    let local = |name, register, ty, mutable, kind| DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(if ty == 1 { 11 } else { 10 }),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind,
        mutable,
        scope: 0,
        declaration: Some(location(1)),
        hidden: false,
        cell_backed: kind == DebugBindingKind::Capture,
        initializer: (name == 3).then_some(InstructionAddress::new(1)),
    };
    let root_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            local(3, 0, 0, true, DebugBindingKind::Local),
            local(4, 1, 0, false, DebugBindingKind::Local),
            local(5, 2, 1, true, DebugBindingKind::Local),
            DebugBinding {
                type_name: StringId::new(12),
                ty: DebugTypeId::new(2),
                ..local(6, 3, 2, true, DebugBindingKind::Local)
            },
            DebugBinding {
                type_name: StringId::new(16),
                ty: DebugTypeId::new(3),
                ..local(7, 4, 3, true, DebugBindingKind::Local)
            },
            local(8, 5, 0, true, DebugBindingKind::Parameter),
            local(9, 6, 0, true, DebugBindingKind::Capture),
        ],
        sequence_points: vec![point(0, 1), point(7, 2)],
        ..Default::default()
    };
    let helper_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![local(8, 0, 0, false, DebugBindingKind::Parameter)],
        sequence_points: vec![point(10, 10)],
        ..Default::default()
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 7, 0).expect("count value"),
            abc(Opcode::Move, 0, 7, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("frozen init"),
            abc(Opcode::LoadUnit, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 7, 2).expect("global value"),
            Instruction::abx(Opcode::StoreGlobal, 7, 0).expect("global store"),
            Instruction::abx(Opcode::LoadConstant, 7, 0).expect("array element"),
            abc(Opcode::MakeArray, 4, 7, 1),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 0, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(10)),
                arity: 0,
                capture_count: 0,
                register_count: 8,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: root_debug,
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(10), InstructionAddress::new(11)),
                arity: 1,
                capture_count: 0,
                register_count: 1,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: helper_debug,
            },
        ],
        constants: vec![
            Constant::Integer(1),
            Constant::Integer(2),
            Constant::Integer(99),
        ],
        strings,
        globals: vec![GlobalInfo {
            name: StringId::new(15),
            ty: DebugTypeId::new(0),
            mutable: true,
            initializer: Some(fpas_bytecode::GlobalInitializer {
                function: FunctionId::new(0),
                instruction: InstructionAddress::new(5),
            }),
        }],
        records: vec![RecordLayout {
            name: StringId::new(12),
            fields: vec![
                RecordField {
                    name: StringId::new(13),
                    ty: DebugTypeId::new(0),
                },
                RecordField {
                    name: StringId::new(14),
                    ty: DebugTypeId::new(0),
                },
            ],
            properties: Vec::new(),
            methods: Vec::new(),
        }],
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![
            DebugType::Integer,
            DebugType::Unit,
            DebugType::Record(RecordTypeId::new(0)),
            DebugType::Array(DebugTypeId::new(0)),
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
                    instruction_start: InstructionAddress::new(7),
                    source: SourceId::new(0),
                    line: 2,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(10),
                    source: SourceId::new(0),
                    line: 10,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("uninitialized assignment executable")
}

pub(super) fn task_assignment_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        ["root", "work", "test.fpas", "Count", "Integer", "Pending"]
            .into_iter()
            .map(str::to_string)
            .collect(),
    );
    let location = |line| fpas_bytecode::DebugSourceLocation {
        source: SourceId::new(0),
        line,
        column: 3,
    };
    let root = FunctionInfo {
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
            bindings: Vec::new(),
            sequence_points: vec![point(0, 1), point(2, 2)],
            ..Default::default()
        },
    };
    let work = FunctionInfo {
        name: StringId::new(1),
        code: CodeRange::new(InstructionAddress::new(4), InstructionAddress::new(6)),
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
            bindings: vec![DebugBinding {
                name: StringId::new(3),
                type_name: StringId::new(4),
                ty: DebugTypeId::new(0),
                register: Register::new(0).expect("register"),
                kind: DebugBindingKind::Local,
                mutable: true,
                scope: 0,
                declaration: Some(location(20)),
                hidden: false,
                cell_backed: false,
                initializer: None,
            }],
            sequence_points: vec![point(4, 20)],
            ..Default::default()
        },
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("work function"),
            abc_aux(Opcode::SpawnTask, 1, 0, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                2,
                u16::from(fpas_bytecode::Intrinsic::Task(
                    fpas_bytecode::TaskIntrinsic::Wait,
                )),
                1,
                1,
            )
            .expect("wait"),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        functions: vec![root, work],
        constants: vec![Constant::Function {
            function: FunctionId::new(1),
            task_bound: false,
        }],
        strings,
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![DebugType::Integer, DebugType::Unit],
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
                    instruction_start: InstructionAddress::new(4),
                    source: SourceId::new(0),
                    line: 20,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("task assignment executable")
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

pub(super) fn named<'a>(items: &'a [crate::DebugVariable], name: &str) -> &'a crate::DebugVariable {
    items
        .iter()
        .find(|item| item.name == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

pub(super) fn root(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.to_string(),
        selectors: Vec::new(),
    }
}

pub(super) fn field(root_name: &str, name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root_name.to_string(),
        selectors: vec![DebugAssignmentSelector::Field(name.to_string())],
    }
}

mod cases;
