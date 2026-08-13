//! Helpers for explicit variant discovery and construction tests.

pub(super) use super::variant_replacement::{
    field, named, root, scope_reference, stop_with_variants, variant_executable,
};
pub(super) use super::*;

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    EnumLayout, EnumTypeId, EnumVariant, Executable, FunctionDebugInfo, FunctionFlags, FunctionId,
    FunctionInfo, GlobalInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode, Register,
    ReturnConvention, SourceId, SourceMap, SourceRun, StringId, StringTable, VerifiedExecutable,
};

mod cases;
mod metadata;

pub(super) fn fields(pairs: &[(&str, DebugExpression)]) -> Vec<(String, DebugExpression)> {
    pairs
        .iter()
        .map(|(name, expression)| ((*name).to_string(), expression.clone()))
        .collect()
}

pub(super) fn index_target(root_name: &str, index: i64) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root_name.to_string(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(
            index,
        ))],
    }
}

pub(super) fn stop_order_session(session: &mut DebugSession) {
    for _ in 0..8 {
        if let Ok(locals) = session
            .stack(0, 1)
            .and_then(|stack| session.scopes(stack.items[0].id))
            && let Some(locals) = locals.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 10)
            && variables
                .items
                .iter()
                .any(|item| item.name == "Selected" && item.value != "<uninitialized>")
        {
            return;
        }
        let _ = stopped(session.step_into().expect("step toward selected"));
    }
    panic!("order fixture Selected never became initialized")
}

pub(super) fn next_call() -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("next".to_string())),
        arguments: Vec::new(),
    }
}

pub(super) fn order_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "next",
            "test.fpas",
            "Selected",
            "Integer",
            "Choice",
            "Pair",
            "Left",
            "Right",
            "Tick",
            "Empty",
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
            type_name: StringId::new(5),
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
        ..Default::default()
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 1, 0).expect("zero"),
            Instruction::abx(Opcode::StoreGlobal, 1, 0).expect("tick"),
            abc(Opcode::MakeEnum, 0, 0, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadGlobal, 0, 0).expect("load tick"),
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("one"),
            abc(Opcode::AddInteger, 0, 0, 1),
            Instruction::abx(Opcode::StoreGlobal, 0, 0).expect("store tick"),
            abc(Opcode::Return, 0, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(4)),
                arity: 0,
                capture_count: 0,
                register_count: 2,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: root_debug,
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(4), InstructionAddress::new(9)),
                arity: 0,
                capture_count: 0,
                register_count: 2,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags::default(),
                debug: FunctionDebugInfo::default(),
            },
        ],
        constants: vec![Constant::Integer(0), Constant::Integer(1)],
        strings,
        globals: vec![GlobalInfo {
            name: StringId::new(9),
            ty: DebugTypeId::new(0),
            mutable: true,
        }],
        records: Vec::new(),
        enums: vec![EnumLayout {
            name: StringId::new(5),
        }],
        enum_variants: vec![
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(10),
                fields: Vec::new(),
                field_types: Vec::new(),
            },
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(6),
                fields: vec![StringId::new(7), StringId::new(8)],
                field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
            },
        ],
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
    .expect("order executable")
}
