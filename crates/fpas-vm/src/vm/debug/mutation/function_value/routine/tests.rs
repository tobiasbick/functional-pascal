use super::{portable_signature, prepare};
use crate::vm::debug::evaluation::DebugEvaluationLimits;
use crate::vm::debug::routines::callable_name_matches;
use crate::vm::debug::types::DebugErrorKind;
use fpas_bytecode::{
    CodeRange, DebugBinding, DebugBindingId, DebugBindingKind, DebugCaptureKind,
    DebugCaptureSource, DebugScope, DebugType, DebugTypeId, Executable, FunctionDebugInfo,
    FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress, Opcode, Register,
    ReturnConvention, SourceId, SourceMap, SourceRun, StringId, StringTable, VerifiedExecutable,
};

fn parameter(register: u16, hidden: bool) -> DebugBinding {
    DebugBinding {
        name: StringId::new(2),
        type_name: StringId::new(3),
        ty: DebugTypeId::new(0),
        register: Register::new(register).expect("register"),
        kind: DebugBindingKind::Parameter,
        mutable: false,
        scope: 0,
        declaration: None,
        hidden,
        cell_backed: false,
        initializer: None,
    }
}

fn function(
    name: StringId,
    arity: u8,
    capture_count: u16,
    bindings: Vec<DebugBinding>,
    result_type: Option<DebugTypeId>,
    start: u32,
) -> FunctionInfo {
    FunctionInfo {
        name,
        code: CodeRange::new(
            InstructionAddress::new(start),
            InstructionAddress::new(start.saturating_add(1)),
        ),
        arity,
        capture_count,
        register_count: 2,
        return_convention: ReturnConvention::Value,
        flags: FunctionFlags::default(),
        debug: FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings,
            sequence_points: Vec::new(),
            result_type,
            ..FunctionDebugInfo::default()
        },
    }
}

fn executable(
    functions: Vec<FunctionInfo>,
    debug_types: Vec<DebugType>,
    names: Vec<&str>,
) -> VerifiedExecutable {
    let mut code = vec![Instruction::abc(Opcode::Return, u16::MAX, 0, 0, 0).expect("root")];
    let mut all_functions = vec![FunctionInfo {
        name: StringId::new(1),
        code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
        arity: 0,
        capture_count: 0,
        register_count: 1,
        return_convention: ReturnConvention::Unit,
        flags: FunctionFlags::default(),
        debug: FunctionDebugInfo::default(),
    }];
    for (index, mut function) in functions.into_iter().enumerate() {
        let start = u32::try_from(index.saturating_add(1)).expect("start");
        function.code = CodeRange::new(
            InstructionAddress::new(start),
            InstructionAddress::new(start.saturating_add(1)),
        );
        code.push(Instruction::abc(Opcode::Return, 0, 0, 0, 0).expect("return"));
        all_functions.push(function);
    }
    attach_cell_capture_provenance(&mut all_functions);
    let runs = all_functions
        .iter()
        .map(|function| SourceRun {
            instruction_start: function.code.start,
            source: SourceId::new(0),
            line: 1,
            column: 3,
        })
        .collect();
    Executable {
        code,
        functions: all_functions,
        constants: Vec::new(),
        strings: StringTable::new(names.into_iter().map(str::to_string).collect()),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types,
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs,
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("routine executable")
}

fn attach_cell_capture_provenance(functions: &mut [FunctionInfo]) {
    let slots = functions
        .iter()
        .map(|function| function.capture_count)
        .max()
        .unwrap_or(0);
    if slots == 0 {
        return;
    }
    functions[0].register_count = functions[0].register_count.max(slots);
    functions[0].debug.scopes = vec![DebugScope {
        id: 0,
        parent: None,
    }];
    functions[0].debug.bindings = (0..slots)
        .map(|index| DebugBinding {
            name: StringId::new(2),
            type_name: StringId::new(3),
            ty: DebugTypeId::new(0),
            register: Register::new(index).expect("register"),
            kind: DebugBindingKind::Local,
            mutable: true,
            scope: 0,
            declaration: None,
            hidden: false,
            cell_backed: true,
            initializer: None,
        })
        .collect();
    for function in functions.iter_mut().skip(1) {
        if function.capture_count == 0 {
            continue;
        }
        function.debug.lexical_owner = Some(FunctionId::new(0));
        function.debug.capture_sources = (0..function.capture_count)
            .map(|index| DebugCaptureSource {
                binding: DebugBindingId::new(u32::from(index)),
                ty: DebugTypeId::new(0),
                kind: DebugCaptureKind::Cell,
            })
            .collect();
    }
}

#[test]
fn complete_metadata_materializes_an_empty_capture_function() {
    let executable = executable(
        vec![function(
            StringId::new(0),
            1,
            0,
            vec![parameter(0, false)],
            Some(DebugTypeId::new(0)),
            0,
        )],
        vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ],
        vec!["addtwo", "test.fpas", "value", "integer"],
    );
    let value = prepare(
        &executable,
        None,
        "AddTwo",
        DebugTypeId::new(1),
        None,
        0,
        None,
        DebugEvaluationLimits::default(),
    )
    .expect("routine");
    match value {
        fpas_bytecode::Value::Function(function) => {
            assert_eq!(function.name, "addtwo");
            assert!(function.captures.is_empty());
            assert!(!function.task_bound);
        }
        other => panic!("expected function, got {}", other.type_name()),
    }
    assert!(callable_name_matches("addtwo", "AddTwo"));
}

#[test]
fn capturing_hidden_unordered_and_missing_metadata_are_rejected() {
    let capturing = executable(
        vec![function(
            StringId::new(0),
            1,
            1,
            vec![parameter(0, false)],
            Some(DebugTypeId::new(0)),
            0,
        )],
        vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ],
        vec!["adder", "test.fpas", "value", "integer"],
    );
    assert_eq!(
        prepare(
            &capturing,
            None,
            "adder",
            DebugTypeId::new(1),
            None,
            0,
            None,
            DebugEvaluationLimits::default()
        )
        .expect_err("captures")
        .kind,
        DebugErrorKind::VariableValueType
    );

    let hidden = executable(
        vec![function(
            StringId::new(0),
            1,
            0,
            vec![parameter(0, true)],
            Some(DebugTypeId::new(0)),
            0,
        )],
        vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ],
        vec!["hidden", "test.fpas", "value", "integer"],
    );
    assert!(
        prepare(
            &hidden,
            None,
            "hidden",
            DebugTypeId::new(1),
            None,
            0,
            None,
            DebugEvaluationLimits::default()
        )
        .expect_err("hidden")
        .message
        .contains("hidden")
    );

    let unordered = executable(
        vec![function(
            StringId::new(0),
            2,
            0,
            vec![parameter(0, false), parameter(0, false)],
            Some(DebugTypeId::new(0)),
            0,
        )],
        vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ],
        vec!["shifted", "test.fpas", "value", "integer"],
    );
    assert!(
        prepare(
            &unordered,
            None,
            "shifted",
            DebugTypeId::new(1),
            None,
            0,
            None,
            DebugEvaluationLimits::default()
        )
        .expect_err("register")
        .message
        .contains("registers")
    );

    let missing_result = executable(
        vec![function(
            StringId::new(0),
            1,
            0,
            vec![parameter(0, false)],
            None,
            0,
        )],
        vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0)],
                result: DebugTypeId::new(0),
            },
        ],
        vec!["incomplete", "test.fpas", "value", "integer"],
    );
    assert!(
        prepare(
            &missing_result,
            None,
            "incomplete",
            DebugTypeId::new(1),
            None,
            0,
            None,
            DebugEvaluationLimits::default()
        )
        .expect_err("result")
        .message
        .contains("result type")
    );
    let image = missing_result.executable();
    assert!(portable_signature(image, &image.functions[1], "incomplete").is_err());
}

#[test]
fn ambiguous_and_unknown_names_are_stable() {
    let executable = executable(
        vec![
            function(
                StringId::new(0),
                0,
                0,
                Vec::new(),
                Some(DebugTypeId::new(0)),
                0,
            ),
            function(
                StringId::new(2),
                0,
                0,
                Vec::new(),
                Some(DebugTypeId::new(0)),
                1,
            ),
        ],
        vec![
            DebugType::Unit,
            DebugType::Function {
                parameters: Vec::new(),
                result: DebugTypeId::new(0),
            },
        ],
        vec!["math.transform", "test.fpas", "stats.transform"],
    );
    assert_eq!(
        prepare(
            &executable,
            None,
            "transform",
            DebugTypeId::new(1),
            None,
            0,
            None,
            DebugEvaluationLimits::default()
        )
        .expect_err("ambiguous")
        .kind,
        DebugErrorKind::AmbiguousCallable
    );
    assert_eq!(
        prepare(
            &executable,
            None,
            "missing",
            DebugTypeId::new(1),
            None,
            0,
            None,
            DebugEvaluationLimits::default()
        )
        .expect_err("unknown")
        .kind,
        DebugErrorKind::UnknownName
    );
    let procedure = prepare(
        &executable,
        None,
        "math.transform",
        DebugTypeId::new(1),
        None,
        0,
        None,
        DebugEvaluationLimits::default(),
    )
    .expect("unique qualified procedure");
    match procedure {
        fpas_bytecode::Value::Function(function) => {
            assert_eq!(function.name, "math.transform");
            assert!(function.captures.is_empty());
        }
        other => panic!("expected function, got {}", other.type_name()),
    }
}
