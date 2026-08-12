//! Hand-built executables and helpers for enum and wrapper replacement tests.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    EnumLayout, EnumTypeId, EnumVariant, Executable, FunctionDebugInfo, FunctionFlags, FunctionId,
    FunctionInfo, GlobalInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode, RecordField,
    RecordLayout, RecordTypeId, Register, ReturnConvention, SourceId, SourceMap, SourceRun,
    StringId, StringTable, VerifiedExecutable,
};

pub(super) use super::*;

pub(super) fn variant_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "helper",
            "test.fpas",
            "Selected",
            "EmptyValue",
            "Outcome",
            "Optional",
            "Missing",
            "Nested",
            "Packed",
            "Holder",
            "Items",
            "Alias",
            "Fixed",
            "Uninit",
            "Value",
            "Integer",
            "Choice",
            "Empty",
            "Count",
            "Pair",
            "Left",
            "Right",
            "Other",
            "Only",
            "Box",
            "Item",
            "G",
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
    let local = |name, register, ty, mutable| DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(16),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind: DebugBindingKind::Local,
        mutable,
        scope: 0,
        declaration: Some(location(1)),
        hidden: false,
        cell_backed: false,
    };
    let root_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            local(3, 0, 2, true),
            local(4, 1, 2, true),
            local(5, 2, 3, true),
            local(6, 3, 4, true),
            local(7, 4, 4, true),
            local(8, 5, 7, true),
            local(9, 6, 6, true),
            local(10, 7, 8, true),
            local(11, 8, 9, true),
            local(12, 9, 2, true),
            local(13, 10, 2, false),
            local(14, 11, 2, true),
        ],
        sequence_points: vec![point(27, 1)],
    };
    let helper_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![DebugBinding {
            name: StringId::new(15),
            type_name: StringId::new(16),
            ty: DebugTypeId::new(0),
            register: Register::new(0).expect("register"),
            kind: DebugBindingKind::Parameter,
            mutable: false,
            scope: 0,
            declaration: Some(location(10)),
            hidden: false,
            cell_backed: false,
        }],
        sequence_points: vec![point(30, 10)],
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 20, 0).expect("1"),
            abc(Opcode::MakeEnum, 0, 1, 20),
            abc(Opcode::MakeEnum, 1, 0, 20),
            Instruction::abx(Opcode::LoadConstant, 20, 1).expect("6"),
            abc(Opcode::MakeOk, 2, 20, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 2).expect("7"),
            abc(Opcode::MakeSome, 3, 20, 0),
            abc(Opcode::MakeNone, 4, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 3).expect("11"),
            abc(Opcode::MakeOk, 5, 20, 0),
            abc(Opcode::MakeSome, 5, 5, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 4).expect("8"),
            Instruction::abx(Opcode::LoadConstant, 21, 5).expect("9"),
            abc(Opcode::MakeArray, 6, 20, 2),
            abc(Opcode::MakeOk, 6, 6, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 6).expect("12"),
            abc(Opcode::MakeEnum, 21, 1, 20),
            abc(Opcode::MakeRecord, 7, 0, 21),
            Instruction::abx(Opcode::LoadConstant, 20, 7).expect("13"),
            abc(Opcode::MakeEnum, 21, 1, 20),
            abc(Opcode::MakeArray, 8, 21, 1),
            abc(Opcode::Move, 9, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 8).expect("99"),
            abc(Opcode::MakeEnum, 10, 1, 20),
            Instruction::abx(Opcode::LoadConstant, 20, 9).expect("15"),
            abc(Opcode::MakeEnum, 21, 1, 20),
            Instruction::abx(Opcode::StoreGlobal, 21, 0).expect("global"),
            abc(Opcode::LoadEnumField, 12, 0, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 12, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(30)),
                arity: 0,
                capture_count: 0,
                register_count: 22,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: root_debug,
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(30), InstructionAddress::new(31)),
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
            Constant::Integer(6),
            Constant::Integer(7),
            Constant::Integer(11),
            Constant::Integer(8),
            Constant::Integer(9),
            Constant::Integer(12),
            Constant::Integer(13),
            Constant::Integer(99),
            Constant::Integer(15),
        ],
        strings,
        globals: vec![GlobalInfo {
            name: StringId::new(27),
            ty: DebugTypeId::new(2),
            mutable: true,
        }],
        records: vec![RecordLayout {
            name: StringId::new(25),
            fields: vec![RecordField {
                name: StringId::new(26),
                ty: DebugTypeId::new(2),
            }],
            properties: Vec::new(),
        }],
        enums: vec![
            EnumLayout {
                name: StringId::new(17),
            },
            EnumLayout {
                name: StringId::new(23),
            },
        ],
        enum_variants: vec![
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(18),
                fields: Vec::new(),
                field_types: Vec::new(),
            },
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(19),
                fields: vec![StringId::new(15)],
                field_types: vec![DebugTypeId::new(0)],
            },
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(20),
                fields: vec![StringId::new(21), StringId::new(22)],
                field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
            },
            EnumVariant {
                owner: EnumTypeId::new(1),
                name: StringId::new(24),
                fields: Vec::new(),
                field_types: Vec::new(),
            },
        ],
        debug_types: vec![
            DebugType::Integer,
            DebugType::String,
            DebugType::Enum(EnumTypeId::new(0)),
            DebugType::Result {
                ok: DebugTypeId::new(0),
                error: DebugTypeId::new(1),
            },
            DebugType::Option(DebugTypeId::new(0)),
            DebugType::Array(DebugTypeId::new(0)),
            DebugType::Result {
                ok: DebugTypeId::new(5),
                error: DebugTypeId::new(1),
            },
            DebugType::Option(DebugTypeId::new(3)),
            DebugType::Record(RecordTypeId::new(0)),
            DebugType::Array(DebugTypeId::new(2)),
            DebugType::Enum(EnumTypeId::new(1)),
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
                    instruction_start: InstructionAddress::new(30),
                    source: SourceId::new(0),
                    line: 10,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("variant executable")
}

pub(super) fn collision_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "Choice.Pair",
            "test.fpas",
            "Selected",
            "Integer",
            "Choice",
            "Pair",
            "Left",
            "Right",
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
    let root_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![DebugBinding {
            name: StringId::new(3),
            type_name: StringId::new(4),
            ty: DebugTypeId::new(1),
            register: Register::new(0).expect("register"),
            kind: DebugBindingKind::Local,
            mutable: true,
            scope: 0,
            declaration: Some(location(1)),
            hidden: false,
            cell_backed: false,
        }],
        sequence_points: vec![point(3, 1)],
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 1, 0).expect("left"),
            Instruction::abx(Opcode::LoadConstant, 2, 0).expect("right"),
            abc(Opcode::MakeEnum, 0, 0, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 2, 1).expect("99"),
            abc(Opcode::Return, 2, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(4)),
                arity: 0,
                capture_count: 0,
                register_count: 3,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: root_debug,
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(4), InstructionAddress::new(6)),
                arity: 2,
                capture_count: 0,
                register_count: 3,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags::default(),
                debug: FunctionDebugInfo::default(),
            },
        ],
        constants: vec![Constant::Integer(1), Constant::Integer(99)],
        strings,
        globals: Vec::new(),
        records: Vec::new(),
        enums: vec![EnumLayout {
            name: StringId::new(5),
        }],
        enum_variants: vec![EnumVariant {
            owner: EnumTypeId::new(0),
            name: StringId::new(6),
            fields: vec![StringId::new(7), StringId::new(8)],
            field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
        }],
        debug_types: vec![DebugType::Integer, DebugType::Enum(EnumTypeId::new(0))],
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
                    line: 10,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("collision executable")
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

pub(super) fn stop_with_variants(session: &mut DebugSession) {
    for _ in 0..40 {
        if let Ok(locals) = session
            .stack(0, 1)
            .and_then(|stack| session.scopes(stack.items[0].id))
            && let Some(locals) = locals.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 20)
            && variables
                .items
                .iter()
                .any(|item| item.name == "Selected" && item.value != "<uninitialized>")
        {
            return;
        }
        let _ = stopped(session.step_into().expect("step toward variants"));
    }
    panic!("variant locals never became initialized")
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

pub(super) fn enum_call(name: &str, arguments: Vec<DebugExpression>) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable(name.to_string())),
        arguments,
    }
}

pub(super) fn fieldless(owner: &str, variant: &str) -> DebugExpression {
    DebugExpression::Field {
        base: Box::new(DebugExpression::Name(owner.to_string())),
        name: variant.to_string(),
    }
}

pub(super) fn pair(left: i64, right: i64) -> DebugExpression {
    enum_call(
        "Choice.Pair",
        vec![
            DebugExpression::Integer(left),
            DebugExpression::Integer(right),
        ],
    )
}

mod cases;
