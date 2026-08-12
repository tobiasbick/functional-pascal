use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    EnumLayout, EnumTypeId, EnumVariant, Executable, FunctionDebugInfo, FunctionFlags, FunctionId,
    FunctionInfo, Instruction, InstructionAddress, NO_REGISTER, Opcode, Register, ReturnConvention,
    SourceId, SourceMap, SourceRun, StringId, StringTable, VerifiedExecutable,
};

use super::*;

fn payload_executable() -> VerifiedExecutable {
    let strings = StringTable::new(
        [
            "root",
            "helper",
            "test.fpas",
            "old",
            "Selected",
            "PairValue",
            "OkValue",
            "ErrorValue",
            "SomeValue",
            "NoneValue",
            "OkItems",
            "Nested",
            "Fixed",
            "Answer",
            "Value",
            "Integer",
            "Choice",
            "Count",
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
    let local = |name, register, ty, mutable| DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(15),
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
            local(4, 0, 2, true),
            local(5, 1, 2, true),
            local(6, 2, 3, true),
            local(7, 3, 3, true),
            local(8, 4, 4, true),
            local(9, 5, 4, true),
            local(10, 6, 6, true),
            local(11, 7, 7, true),
            local(12, 8, 2, false),
            local(13, 9, 0, true),
        ],
        sequence_points: vec![point(21, 1)],
    };
    let helper_debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![DebugBinding {
            name: StringId::new(14),
            type_name: StringId::new(15),
            ty: DebugTypeId::new(0),
            register: Register::new(0).expect("register"),
            kind: DebugBindingKind::Parameter,
            mutable: false,
            scope: 0,
            declaration: Some(location(10)),
            hidden: false,
            cell_backed: false,
        }],
        sequence_points: vec![point(24, 10)],
    };
    Executable {
        code: vec![
            Instruction::abx(Opcode::LoadConstant, 20, 0).expect("1"),
            abc(Opcode::MakeEnum, 0, 0, 20),
            Instruction::abx(Opcode::LoadConstant, 20, 1).expect("2"),
            Instruction::abx(Opcode::LoadConstant, 21, 2).expect("3"),
            abc(Opcode::MakeEnum, 1, 1, 20),
            Instruction::abx(Opcode::LoadConstant, 20, 3).expect("6"),
            abc(Opcode::MakeOk, 2, 20, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 4).expect("old"),
            abc(Opcode::MakeError, 3, 20, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 5).expect("7"),
            abc(Opcode::MakeSome, 4, 20, 0),
            abc(Opcode::MakeNone, 5, 0, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 6).expect("8"),
            Instruction::abx(Opcode::LoadConstant, 21, 7).expect("9"),
            abc(Opcode::MakeArray, 6, 20, 2),
            abc(Opcode::MakeOk, 6, 6, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 8).expect("11"),
            abc(Opcode::MakeOk, 7, 20, 0),
            abc(Opcode::MakeSome, 7, 7, 0),
            Instruction::abx(Opcode::LoadConstant, 20, 0).expect("fixed 1"),
            abc(Opcode::MakeEnum, 8, 0, 20),
            abc(Opcode::LoadEnumField, 9, 0, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 9, 1),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(24)),
                arity: 0,
                capture_count: 0,
                register_count: 22,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: root_debug,
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(24), InstructionAddress::new(25)),
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
            Constant::Integer(3),
            Constant::Integer(6),
            Constant::String(StringId::new(3)),
            Constant::Integer(7),
            Constant::Integer(8),
            Constant::Integer(9),
            Constant::Integer(11),
        ],
        strings,
        globals: Vec::new(),
        records: Vec::new(),
        enums: vec![EnumLayout {
            name: StringId::new(16),
        }],
        enum_variants: vec![
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(17),
                fields: vec![StringId::new(14)],
                field_types: vec![DebugTypeId::new(0)],
            },
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(18),
                fields: vec![StringId::new(19), StringId::new(20)],
                field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
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
                    instruction_start: InstructionAddress::new(24),
                    source: SourceId::new(0),
                    line: 10,
                    column: 3,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("payload executable")
}

fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}

fn stop_with_payloads(session: &mut DebugSession) {
    for _ in 0..32 {
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
        let _ = stopped(session.step_into().expect("step toward payloads"));
    }
    panic!("payload locals never became initialized")
}

fn named<'a>(items: &'a [crate::DebugVariable], name: &str) -> &'a crate::DebugVariable {
    items
        .iter()
        .find(|item| item.name == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn field(root: &str, name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root.to_string(),
        selectors: vec![DebugAssignmentSelector::Field(name.to_string())],
    }
}

#[test]
fn handle_and_textual_payload_updates_commit_and_continue() {
    let mut session = DebugSession::new(payload_executable()).expect("debug session");
    stop_with_payloads(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    session
        .set_variable(
            named(&variables.items, "Selected").variables_reference,
            "Value",
            &DebugExpression::Integer(10),
        )
        .expect("enum field");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("fresh locals");
    session
        .set_variable(
            named(&variables.items, "PairValue").variables_reference,
            "Right",
            &DebugExpression::Integer(30),
        )
        .expect("second enum field");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("fresh locals");
    session
        .set_variable(
            named(&variables.items, "OkValue").variables_reference,
            "value",
            &DebugExpression::Integer(20),
        )
        .expect("ok payload");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("fresh locals");
    session
        .set_variable(
            named(&variables.items, "ErrorValue").variables_reference,
            "VaLuE",
            &DebugExpression::String("new".to_string()),
        )
        .expect("error payload");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("fresh locals");
    session
        .set_variable(
            named(&variables.items, "SomeValue").variables_reference,
            "value",
            &DebugExpression::Integer(70),
        )
        .expect("some payload");

    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "OkItems".to_string(),
                selectors: vec![
                    DebugAssignmentSelector::Field("value".to_string()),
                    DebugAssignmentSelector::Index(DebugExpression::Integer(1)),
                ],
            },
            &DebugExpression::Integer(90),
            Some(frame),
        )
        .expect("nested array payload");
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Nested".to_string(),
                selectors: vec![
                    DebugAssignmentSelector::Field("value".to_string()),
                    DebugAssignmentSelector::Field("value".to_string()),
                ],
            },
            &DebugExpression::Integer(42),
            Some(frame),
        )
        .expect("nested wrapper payload");

    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("committed locals");
    let selected_fields = session
        .variables(
            named(&variables.items, "Selected").variables_reference,
            0,
            10,
        )
        .expect("selected fields");
    assert_eq!(named(&selected_fields.items, "Value").value, "10");
    let pair_fields = session
        .variables(
            named(&variables.items, "PairValue").variables_reference,
            0,
            10,
        )
        .expect("pair fields");
    assert_eq!(named(&pair_fields.items, "Left").value, "2");
    assert_eq!(named(&pair_fields.items, "Right").value, "30");
    let ok_fields = session
        .variables(
            named(&variables.items, "OkValue").variables_reference,
            0,
            10,
        )
        .expect("ok fields");
    assert_eq!(named(&ok_fields.items, "value").value, "20");

    stopped(session.step_into().expect("load mutated field"));
    for _ in 0..8 {
        if scope_reference_opt(&mut session, "Parameters").is_some() {
            let parameters = scope_reference(&mut session, "Parameters");
            assert_eq!(
                session
                    .variables(parameters, 0, 1)
                    .expect("helper parameter")
                    .items[0]
                    .value,
                "10"
            );
            return;
        }
        let _ = stopped(session.step_into().expect("enter helper"));
    }
    panic!("helper never received the mutated payload");
}

fn scope_reference_opt(session: &mut DebugSession, scope_name: &str) -> Option<u64> {
    let frame = session.stack(0, 1).ok()?.items.first()?.id;
    session
        .scopes(frame)
        .ok()?
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .map(|scope| scope.variables_reference)
}

#[test]
fn payload_failures_are_atomic_and_preserve_existing_handles() {
    let mut session = DebugSession::new(payload_executable()).expect("debug session");
    stop_with_payloads(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");

    assert_eq!(
        session
            .set_expression(
                &field("NoneValue", "value"),
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("none payload")
            .kind,
        DebugErrorKind::VariablePathUnsupported
    );
    assert_eq!(
        session
            .set_expression(
                &field("Selected", "Left"),
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("unknown enum field")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    assert_eq!(
        session
            .set_expression(
                &field("OkValue", "count"),
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("unknown wrapper child")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    assert_eq!(
        session
            .set_expression(
                &field("Selected", "Value"),
                &DebugExpression::String("wrong".to_string()),
                Some(frame),
            )
            .expect_err("wrong type")
            .kind,
        DebugErrorKind::VariableValueType
    );
    assert_eq!(
        session
            .set_expression(
                &field("Fixed", "Value"),
                &DebugExpression::Integer(1),
                Some(frame),
            )
            .expect_err("immutable root")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
    assert_eq!(
        session
            .set_variable(locals, "MissingChild", &DebugExpression::Integer(1))
            .expect_err("unknown child")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    assert_eq!(
        session
            .set_expression_with_limits(
                &field("Selected", "Value"),
                &DebugExpression::Binary {
                    operation: DebugBinaryOperation::Add,
                    left: Box::new(DebugExpression::Integer(1)),
                    right: Box::new(DebugExpression::Integer(2)),
                },
                Some(frame),
                DebugEvaluationLimits {
                    max_operations: 1,
                    ..DebugEvaluationLimits::default()
                },
            )
            .expect_err("evaluation limit")
            .kind,
        DebugErrorKind::EvaluationLimit
    );
    assert!(session.scopes(frame).is_ok(), "failures preserve frames");
    let locals = scope_reference(&mut session, "Locals");
    let variables = session.variables(locals, 0, 20).expect("locals");
    let selected = session
        .variables(
            named(&variables.items, "Selected").variables_reference,
            0,
            10,
        )
        .expect("unchanged selected");
    assert_eq!(named(&selected.items, "Value").value, "1");

    session
        .set_expression(
            &field("Selected", "Value"),
            &DebugExpression::Integer(10),
            Some(frame),
        )
        .expect("successful write");
    assert_eq!(
        session
            .set_expression(
                &field("Selected", "Value"),
                &DebugExpression::Integer(11),
                Some(frame),
            )
            .expect_err("stale frame")
            .kind,
        DebugErrorKind::UnknownFrame
    );
}
