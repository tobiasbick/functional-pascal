//! Hand-built executables for debugger function-value assignment.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingId, DebugBindingKind, DebugCaptureKind,
    DebugCaptureSource, DebugScope, DebugType, DebugTypeId, Executable, FunctionDebugInfo,
    FunctionFlags, FunctionId, FunctionInfo, GlobalInfo, Instruction, InstructionAddress,
    NO_REGISTER, Opcode, RecordField, RecordLayout, RecordMethod, Register, ReturnConvention,
    SourceId, SourceMap, SourceRun, StringId, StringTable, VerifiedExecutable,
};

pub(super) use super::*;

pub(super) fn assignment_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "helper",
            "add_one",
            "add_two",
            "adder",
            "nested",
            "test.fpas",
            "Current",
            "Backup",
            "Captured",
            "Frozen",
            "Number",
            "Loose",
            "Bound",
            "Box",
            "Items",
            "Scores",
            "Optional",
            "Missing",
            "CellSlot",
            "NestedCell",
            "Shared",
            "Hidden",
            "Wrong",
            "Handler",
            "Integer",
            "Dynamic",
            "Holder",
            "Callback",
            "a",
            "Arg",
            "G",
            "Value",
            "Math.Transform",
            "Stats.Transform",
            "backup",
            "Holder.Add",
            "Self",
            "Add",
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
        type_name: StringId::new(match ty {
            0 => 25,
            9 => 26,
            _ => 24,
        }),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind,
        mutable,
        scope: 0,
        declaration: Some(location(1)),
        hidden,
        cell_backed: kind == DebugBindingKind::Capture,
        initializer: None,
    };
    let root_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            local(7, 0, 2, true, DebugBindingKind::Local, false),
            local(8, 1, 2, false, DebugBindingKind::Local, false),
            local(9, 2, 2, false, DebugBindingKind::Local, false),
            local(10, 3, 2, false, DebugBindingKind::Local, false),
            local(11, 4, 0, true, DebugBindingKind::Local, false),
            local(12, 5, 9, true, DebugBindingKind::Local, false),
            local(13, 6, 2, true, DebugBindingKind::Local, false),
            DebugBinding {
                type_name: StringId::new(27),
                ty: DebugTypeId::new(7),
                ..local(14, 7, 7, true, DebugBindingKind::Local, false)
            },
            local(15, 8, 4, true, DebugBindingKind::Local, false),
            local(16, 9, 6, true, DebugBindingKind::Local, false),
            local(17, 10, 8, true, DebugBindingKind::Local, false),
            local(18, 11, 8, true, DebugBindingKind::Local, false),
            local(19, 12, 2, true, DebugBindingKind::Capture, false),
            local(20, 13, 2, true, DebugBindingKind::Local, false),
            local(21, 14, 2, true, DebugBindingKind::Local, false),
            local(22, 15, 2, true, DebugBindingKind::Local, true),
            local(23, 16, 3, true, DebugBindingKind::Local, false),
        ],
        sequence_points: vec![point(0, 1), point(24, 2), point(26, 3)],
        ..Default::default()
    };
    let helper_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![local(30, 0, 2, true, DebugBindingKind::Parameter, false)],
        sequence_points: vec![point(37, 10)],
        result_type: Some(DebugTypeId::new(1)),
        ..Default::default()
    };
    let integer_param = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![local(32, 0, 0, false, DebugBindingKind::Parameter, false)],
        result_type: Some(DebugTypeId::new(0)),
        ..Default::default()
    };
    let method_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![
            local(37, 0, 7, false, DebugBindingKind::Parameter, false),
            local(32, 1, 0, false, DebugBindingKind::Parameter, false),
        ],
        result_type: Some(DebugTypeId::new(0)),
        ..Default::default()
    };
    let routine = |name, start, end, arity, captures, registers, convention, debug| FunctionInfo {
        name: StringId::new(name),
        code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
        arity,
        capture_count: captures,
        register_count: registers,
        return_convention: convention,
        flags: FunctionFlags::default(),
        debug,
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 1, 1).expect("Backup"),
            Instruction::abx(Opcode::LoadConstant, 17, 0).expect("add_one"),
            abc(Opcode::Move, 3, 17, 0),
            Instruction::abx(Opcode::LoadConstant, 4, 7).expect("Number"),
            abc(Opcode::Move, 5, 17, 0),
            Instruction::abx(Opcode::LoadConstant, 6, 0).expect("Bound placeholder"),
            Instruction::abx(Opcode::LoadConstant, 14, 1).expect("Shared local"),
            abc(Opcode::Move, 15, 17, 0),
            Instruction::abx(Opcode::LoadConstant, 16, 0).expect("Wrong"),
            Instruction::abx(Opcode::LoadConstant, 18, 4).expect("10"),
            abc_aux(Opcode::MakeClosure, 2, 3, 18, 1),
            abc(Opcode::MakeRecord, 7, 0, 17),
            abc(Opcode::MakeArray, 8, 17, 1),
            Instruction::abx(Opcode::LoadConstant, 18, 6).expect("key"),
            abc(Opcode::Move, 19, 17, 0),
            abc(Opcode::MakeDictionary, 9, 18, 1),
            abc(Opcode::MakeSome, 10, 17, 0),
            abc(Opcode::MakeNone, 11, 0, 0),
            abc(Opcode::MakeCell, 12, 17, 0),
            Instruction::abx(Opcode::LoadConstant, 18, 2).expect("cell int"),
            abc(Opcode::MakeCell, 19, 18, 0),
            abc(Opcode::MakeArray, 20, 19, 1),
            abc_aux(Opcode::MakeClosure, 13, 4, 20, 1),
            abc_aux(Opcode::MakeClosure, 6, 3, 19, 1),
            Instruction::abx(Opcode::LoadConstant, 0, 0).expect("Current"),
            Instruction::abx(Opcode::StoreGlobal, 1, 0).expect("global G"),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 5, 0, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 2).expect("add_one +1"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 3).expect("add_two +2"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 8).expect("transform +3"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 9).expect("transform +4"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 1, 10).expect("backup +100"),
            abc(Opcode::AddInteger, 2, 0, 1),
            abc(Opcode::Return, 2, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 2, 8).expect("method +3"),
            abc(Opcode::AddInteger, 3, 1, 2),
            abc(Opcode::Return, 3, 0, 0),
        ],
        functions: vec![
            routine(0, 0, 28, 0, 0, 24, ReturnConvention::Unit, root_debug),
            routine(
                2,
                28,
                31,
                1,
                0,
                3,
                ReturnConvention::Value,
                integer_param.clone(),
            ),
            routine(
                3,
                31,
                34,
                1,
                0,
                3,
                ReturnConvention::Value,
                integer_param.clone(),
            ),
            routine(
                4,
                34,
                36,
                1,
                1,
                3,
                ReturnConvention::Value,
                capturing_cell_debug(12, 2),
            ),
            routine(
                5,
                36,
                37,
                0,
                1,
                1,
                ReturnConvention::Unit,
                capturing_cell_debug(12, 2),
            ),
            routine(1, 37, 38, 1, 0, 1, ReturnConvention::Unit, helper_debug),
            routine(
                33,
                38,
                41,
                1,
                0,
                3,
                ReturnConvention::Value,
                integer_param.clone(),
            ),
            routine(
                34,
                41,
                44,
                1,
                0,
                3,
                ReturnConvention::Value,
                integer_param.clone(),
            ),
            routine(35, 44, 47, 1, 0, 3, ReturnConvention::Value, integer_param),
            routine(36, 47, 50, 2, 0, 4, ReturnConvention::Value, method_debug),
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
            Constant::Integer(1),
            Constant::Integer(2),
            Constant::Integer(10),
            Constant::Integer(0),
            Constant::String(StringId::new(29)),
            Constant::Integer(7),
            Constant::Integer(3),
            Constant::Integer(4),
            Constant::Integer(100),
        ],
        strings,
        globals: vec![GlobalInfo {
            name: StringId::new(31),
            ty: DebugTypeId::new(2),
            mutable: true,
            initializer: None,
        }],
        records: vec![RecordLayout {
            name: StringId::new(27),
            fields: vec![RecordField {
                name: StringId::new(28),
                ty: DebugTypeId::new(2),
            }],
            properties: Vec::new(),
            methods: vec![RecordMethod {
                name: StringId::new(38),
                routine: StringId::new(36),
            }],
        }],
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![
            DebugType::Integer,
            DebugType::Unit,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
            DebugType::Function {
                parameters: Vec::new(),
                result: DebugTypeId::new(0),
            },
            DebugType::Array(DebugTypeId::new(2)),
            DebugType::String,
            DebugType::Dictionary {
                key: DebugTypeId::new(5),
                value: DebugTypeId::new(2),
            },
            DebugType::Record(fpas_bytecode::RecordTypeId::new(0)),
            DebugType::Option(DebugTypeId::new(2)),
            DebugType::Dynamic,
        ],
        source_map: SourceMap {
            sources: vec![StringId::new(6)],
            runs: vec![
                SourceRun {
                    instruction_start: InstructionAddress::new(0),
                    source: SourceId::new(0),
                    line: 1,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(28),
                    source: SourceId::new(0),
                    line: 20,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(31),
                    source: SourceId::new(0),
                    line: 21,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(34),
                    source: SourceId::new(0),
                    line: 22,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(36),
                    source: SourceId::new(0),
                    line: 23,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(37),
                    source: SourceId::new(0),
                    line: 10,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(38),
                    source: SourceId::new(0),
                    line: 24,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(41),
                    source: SourceId::new(0),
                    line: 25,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(44),
                    source: SourceId::new(0),
                    line: 26,
                    column: 3,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(47),
                    source: SourceId::new(0),
                    line: 27,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("function-value assignment executable")
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

pub(super) fn name(name: &str) -> DebugExpression {
    DebugExpression::Name(name.to_string())
}

pub(super) fn stop_before_current(session: &mut DebugSession) {
    for _ in 0..32 {
        if let Ok(locals) = session
            .stack(0, 1)
            .and_then(|stack| session.scopes(stack.items[0].id))
            && let Some(locals) = locals.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 30)
            && named(&variables.items, "Backup")
                .value
                .starts_with("<function")
            && named(&variables.items, "Current").value == "<uninitialized>"
        {
            return;
        }
        let _ = stopped(
            session
                .step_into()
                .expect("step toward uninitialized Current"),
        );
    }
    panic!("Backup never initialized while Current stayed empty")
}

pub(super) fn stop_with_functions(session: &mut DebugSession) {
    for _ in 0..32 {
        if let Ok(locals) = session
            .stack(0, 1)
            .and_then(|stack| session.scopes(stack.items[0].id))
            && let Some(locals) = locals.into_iter().find(|scope| scope.name == "Locals")
            && let Ok(variables) = session.variables(locals.variables_reference, 0, 30)
            && named(&variables.items, "Current")
                .value
                .starts_with("<function")
        {
            return;
        }
        let _ = stopped(
            session
                .step_into()
                .expect("step toward initialized functions"),
        );
    }
    panic!("Current never became an initialized function")
}

#[test]
fn compiled_fixture_retains_portable_routine_parameter_and_result_metadata() {
    const SOURCE: &str =
        include_str!("../../../../../../../tests/debugger/fixtures/function_value_assignment.fpas");
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile function-value fixture");
    let image = executable.executable();
    let add_two = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("addtwo"))
        .expect("AddTwo");
    assert_eq!(add_two.arity, 1);
    assert_eq!(add_two.capture_count, 0);
    let parameters = add_two
        .debug
        .bindings
        .iter()
        .filter(|binding| binding.kind == DebugBindingKind::Parameter && !binding.hidden)
        .collect::<Vec<_>>();
    assert_eq!(parameters.len(), 1);
    assert!(
        parameters[0].register.get() < add_two.register_count,
        "parameter register must be inside the function frame"
    );
    assert_eq!(
        image.debug_types.get(parameters[0].ty.get() as usize),
        Some(&DebugType::Integer)
    );
    assert_eq!(
        add_two
            .debug
            .result_type
            .and_then(|ty| image.debug_types.get(ty.get() as usize)),
        Some(&DebugType::Integer)
    );
    let transform = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("math.transform"))
        .expect("Math.Transform");
    assert_eq!(transform.arity, 1);
    assert_eq!(transform.capture_count, 0);
    let transform_parameters = transform
        .debug
        .bindings
        .iter()
        .filter(|binding| binding.kind == DebugBindingKind::Parameter && !binding.hidden)
        .collect::<Vec<_>>();
    assert_eq!(transform_parameters.len(), 1);
    assert!(
        transform_parameters[0].register.get() < transform.register_count,
        "static method parameter register must be inside the function frame"
    );
}

fn capturing_cell_debug(binding: u32, ty: u32) -> FunctionDebugInfo {
    FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        lexical_owner: Some(FunctionId::new(0)),
        capture_sources: vec![DebugCaptureSource {
            binding: DebugBindingId::new(binding),
            ty: DebugTypeId::new(ty),
            kind: DebugCaptureKind::Cell,
        }],
        ..Default::default()
    }
}

pub(super) fn function_identity(left: &fpas_bytecode::Value, right: &fpas_bytecode::Value) -> bool {
    match (left, right) {
        (fpas_bytecode::Value::Function(first), fpas_bytecode::Value::Function(second)) => {
            std::ptr::eq(&**first, &**second)
        }
        _ => false,
    }
}

mod capture_destination;
mod cases;
mod dynamic;
mod opaque;
mod reconstruction;
mod routines;
