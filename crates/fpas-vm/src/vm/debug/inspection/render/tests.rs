use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use fpas_bytecode::{
    EnumTypeId, EnumVariantId, FunctionId, RecordTypeId, RuntimeEnumLayout, RuntimeRecordLayout,
    SharedArray, SharedEnum, SharedRecord, Value,
};

use super::{DebugInspectionLimits, RetainedValue, render};

fn retained(value: Value) -> RetainedValue {
    RetainedValue {
        name: "value".to_string(),
        type_name: value.type_name().to_string(),
        value: Some(value),
        presentation_hint: None,
        depth: 0,
        visited_cells: HashSet::new(),
        debug_type: None,
        mutation: super::MutationAccess::NotMutable,
    }
}

#[test]
fn summaries_cover_scalars_wrappers_aggregates_functions_and_opaque_values() {
    let limits = DebugInspectionLimits {
        max_children: 2,
        max_string_chars: 3,
        ..DebugInspectionLimits::default()
    };
    assert_eq!(render(&retained(Value::Integer(7)), limits).summary, "7");
    assert_eq!(
        render(&retained(Value::Real(f64::NAN)), limits).summary,
        "NaN"
    );
    assert_eq!(
        render(&retained(Value::Real(f64::INFINITY)), limits).summary,
        "Infinity"
    );
    assert_eq!(
        render(&retained(Value::Str("abcdef".into())), limits).summary,
        "'abc…'"
    );
    assert_eq!(
        render(&retained(Value::OpaqueHandle(4)), limits).presentation_hint,
        Some("opaque".to_string())
    );

    let array = render(
        &retained(Value::Array(SharedArray::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))),
        limits,
    );
    assert_eq!(array.summary, "[3 items]");
    assert_eq!(array.indexed_children, 3);
    assert_eq!(array.children.len(), 2);
    assert_eq!(array.presentation_hint, Some("truncated".to_string()));

    let mut deep = retained(Value::Array(SharedArray::from(vec![Value::Integer(1)])));
    deep.depth = limits.max_depth;
    assert_eq!(render(&deep, limits).summary, "<max depth>");

    let dictionary = render(
        &retained(Value::dict(vec![(
            Value::Str("key".into()),
            Value::Boolean(true),
        )])),
        limits,
    );
    assert_eq!(dictionary.named_children, 2);

    let record = Value::Record(SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(0),
            type_name: "Point".to_string(),
            fields: vec!["x".to_string()],
        }),
        vec![Value::Integer(1)],
    ));
    assert_eq!(render(&retained(record), limits).summary, "Point {...}");

    let enumeration = Value::Enum(SharedEnum::new(
        Arc::new(RuntimeEnumLayout {
            enumeration: EnumTypeId::new(0),
            variant_id: EnumVariantId::new(0),
            type_name: "Choice".to_string(),
            variant: "Some".to_string(),
            fields: vec!["value".to_string()],
        }),
        vec![Value::Integer(1)],
    ));
    assert_eq!(
        render(&retained(enumeration), limits).summary,
        "Choice.Some"
    );

    for wrapper in [
        Value::result_ok(Value::Integer(1)),
        Value::result_error(Value::Str("error".into())),
        Value::option_some(Value::Boolean(true)),
    ] {
        assert_eq!(render(&retained(wrapper), limits).named_children, 1);
    }
    assert_eq!(render(&retained(Value::OptionNone), limits).summary, "None");

    let function = render(
        &retained(Value::function(
            FunctionId::new(0),
            "callback".to_string(),
            vec![Value::Integer(1)],
        )),
        limits,
    );
    assert_eq!(function.summary, "<function callback>");
    assert_eq!(function.named_children, 1);
    assert_eq!(
        render(&retained(Value::Task(9)), limits).summary,
        "<task 9>"
    );
}

#[test]
fn cells_report_cycles_contention_and_poisoning_without_blocking() {
    let limits = DebugInspectionLimits::default();
    let cycle = Arc::new(Mutex::new(Value::Unit));
    *cycle.lock().expect("cycle lock") = Value::Cell(Arc::clone(&cycle));
    let first = render(&retained(Value::Cell(Arc::clone(&cycle))), limits);
    let second = render(&first.children[0], limits);
    assert_eq!(second.summary, "<cycle>");
    assert_eq!(second.presentation_hint, Some("cycle".to_string()));

    let busy = Arc::new(Mutex::new(Value::Integer(1)));
    let guard = busy.lock().expect("busy lock");
    assert_eq!(
        render(&retained(Value::Cell(Arc::clone(&busy))), limits).summary,
        "<cell busy>"
    );
    drop(guard);

    let poisoned = Arc::new(Mutex::new(Value::Integer(2)));
    let thread_cell = Arc::clone(&poisoned);
    assert!(
        std::thread::spawn(move || {
            let _guard = thread_cell.lock().expect("poison lock");
            panic!("poison debugger test cell");
        })
        .join()
        .is_err()
    );
    assert_eq!(
        render(&retained(Value::Cell(poisoned)), limits).presentation_hint,
        Some("captured mutable, poisoned".to_string())
    );
}

fn writable(value: Value, ty: fpas_bytecode::DebugTypeId) -> RetainedValue {
    let mut retained = retained(value);
    retained.debug_type = Some(ty);
    retained.mutation = super::MutationAccess::Writable(super::super::targets::MutationTarget {
        root: super::super::targets::MutationRoot::FrameRegister(0),
        path: Vec::new(),
        expected_type: ty,
        generation: 1,
        frame_id: Some(1),
        initialized: true,
        initializer: None,
    });
    retained
}

fn payload_executable() -> fpas_bytecode::Executable {
    use fpas_bytecode::{
        CodeRange, DebugType, DebugTypeId, EnumLayout, EnumVariant, Executable, FunctionFlags,
        FunctionInfo, Instruction, InstructionAddress, Opcode, ReturnConvention, SourceId,
        SourceMap, SourceRun, StringId, StringTable,
    };
    Executable {
        code: vec![
            Instruction::abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0).expect("return"),
        ],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        }],
        constants: Vec::new(),
        strings: StringTable::new(vec!["root".to_string(), "test.fpas".to_string()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: vec![EnumLayout {
            name: StringId::new(0),
        }],
        enum_variants: vec![
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(0),
                fields: vec![StringId::new(0)],
                field_types: vec![DebugTypeId::new(0)],
            },
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(0),
                fields: vec![StringId::new(0), StringId::new(0)],
                field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
            },
        ],
        debug_types: vec![
            fpas_bytecode::DebugType::Integer,
            fpas_bytecode::DebugType::String,
            DebugType::Enum(EnumTypeId::new(0)),
            DebugType::Result {
                ok: DebugTypeId::new(0),
                error: DebugTypeId::new(1),
            },
            DebugType::Option(DebugTypeId::new(0)),
        ],
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 1,
            }],
        },
        entry: FunctionId::new(0),
    }
}

fn is_writable(value: &RetainedValue) -> bool {
    matches!(value.mutation, super::MutationAccess::Writable(_))
}

#[test]
fn payload_children_are_writable_only_through_writable_typed_roots() {
    use fpas_bytecode::DebugTypeId;

    let limits = DebugInspectionLimits::default();
    let executable = payload_executable();
    let enumeration = Value::Enum(SharedEnum::new(
        Arc::new(RuntimeEnumLayout {
            enumeration: EnumTypeId::new(0),
            variant_id: EnumVariantId::new(1),
            type_name: "Choice".to_string(),
            variant: "Pair".to_string(),
            fields: vec!["Left".to_string(), "Right".to_string()],
        }),
        vec![Value::Integer(1), Value::Integer(2)],
    ));
    let rendered = super::render_with_executable(
        &writable(enumeration, DebugTypeId::new(2)),
        limits,
        Some(&executable),
    );
    assert_eq!(
        rendered
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect::<Vec<_>>(),
        ["Left", "Right"]
    );
    assert!(rendered.children.iter().all(is_writable));
    assert_eq!(rendered.children[1].debug_type, Some(DebugTypeId::new(0)));

    let ok = super::render_with_executable(
        &writable(Value::result_ok(Value::Integer(6)), DebugTypeId::new(3)),
        limits,
        Some(&executable),
    );
    assert_eq!(ok.children[0].name, "value");
    assert!(is_writable(&ok.children[0]));
    assert_eq!(ok.children[0].debug_type, Some(DebugTypeId::new(0)));

    let error = super::render_with_executable(
        &writable(
            Value::result_error(Value::Str("old".into())),
            DebugTypeId::new(3),
        ),
        limits,
        Some(&executable),
    );
    assert!(is_writable(&error.children[0]));
    assert_eq!(error.children[0].debug_type, Some(DebugTypeId::new(1)));

    let some = super::render_with_executable(
        &writable(Value::option_some(Value::Integer(7)), DebugTypeId::new(4)),
        limits,
        Some(&executable),
    );
    assert!(is_writable(&some.children[0]));

    let none = super::render_with_executable(
        &writable(Value::OptionNone, DebugTypeId::new(4)),
        limits,
        Some(&executable),
    );
    assert!(none.children.is_empty());

    let immutable = super::render_with_executable(
        &{
            let mut value = writable(Value::option_some(Value::Integer(7)), DebugTypeId::new(4));
            value.mutation = super::MutationAccess::NotMutable;
            value
        },
        limits,
        Some(&executable),
    );
    assert!(matches!(
        immutable.children[0].mutation,
        super::MutationAccess::NotMutable
    ));
    assert_eq!(immutable.children[0].name, "value");
}
