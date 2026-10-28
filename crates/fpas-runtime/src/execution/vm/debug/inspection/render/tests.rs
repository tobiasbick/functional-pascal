use std::collections::HashSet;
use std::sync::Arc;

use crate::{
    EnumTypeId, EnumVariantId, FunctionId, MemoryDomain, RecordTypeId, RuntimeEnumLayout,
    RuntimeRecordLayout, SharedArray, SharedEnum, SharedRecord, Value,
};

use super::{DebugInspectionLimits, RetainedValue, render};

fn retained(value: Value) -> RetainedValue {
    RetainedValue {
        domain: None,
        name: "value".to_string(),
        type_name: value.type_name().to_string(),
        value: Some(value),
        presentation_hint: None,
        depth: 0,
        visited_cells: HashSet::new(),
        debug_type: None,
        origin: super::MutationAccess::NotMutable,
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
            error: None,
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
            payload_executable()
                .verify()
                .expect("verified payload image")
                .function_target(FunctionId::new(0))
                .expect("root target"),
            vec![Value::Integer(1)],
        )),
        limits,
    );
    assert_eq!(function.summary, "<function root>");
    assert_eq!(function.named_children, 1);
    assert_eq!(
        render(&retained(Value::Task(9)), limits).summary,
        "<task 9>"
    );
}

#[test]
fn cells_report_cycles_and_inaccessible_ownership_without_blocking() {
    let limits = DebugInspectionLimits::default();
    let domain = MemoryDomain::new();
    let cycle = domain.new_cell(Value::Unit).expect("cycle cell");
    cycle
        .replace(&domain, Value::Cell(cycle.clone()))
        .expect("create cycle");
    let mut value = retained(Value::Cell(cycle.clone()));
    value.domain = Some(domain.clone());
    let first = render(&value, limits);
    let second = render(&first.children[0], limits);
    assert_eq!(second.summary, "<cycle>");
    assert_eq!(second.presentation_hint, Some("cycle".to_string()));
    cycle.replace(&domain, Value::Unit).expect("break cycle");

    let cell = domain.new_cell(Value::Integer(1)).expect("owned cell");
    let mut inaccessible = retained(Value::Cell(cell));
    assert_eq!(render(&inaccessible, limits).summary, "<cell unavailable>");
    inaccessible.domain = Some(MemoryDomain::new());
    let denied = render(&inaccessible, limits);
    assert_eq!(denied.summary, "<cell unavailable>");
    assert_eq!(denied.presentation_hint, Some("unavailable".to_string()));
}

fn writable(value: Value, ty: crate::DebugTypeId) -> RetainedValue {
    let mut retained = retained(value);
    retained.debug_type = Some(ty);
    retained.origin = super::MutationAccess::Writable(super::super::targets::MutationTarget {
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

fn payload_executable() -> crate::Executable {
    use crate::{
        CodeRange, DebugType, DebugTypeId, EnumLayout, EnumVariant, Executable, FunctionFlags,
        FunctionInfo, Instruction, InstructionAddress, Opcode, ReturnConvention, SourceId,
        SourceMap, SourceRun, StringId, StringTable,
    };
    Executable {
        code: vec![Instruction::abc(Opcode::Return, crate::NO_REGISTER, 0, 0, 0).expect("return")],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: crate::FunctionDebugInfo::default(),
        }],
        constants: Vec::new(),
        strings: StringTable::new(vec!["root".to_string(), "test.fpas".to_string()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: vec![EnumLayout {
            error: None,
            name: StringId::new(0),
        }],
        enum_variants: vec![
            EnumVariant {
                error: None,
                owner: EnumTypeId::new(0),
                name: StringId::new(0),
                fields: vec![StringId::new(0)],
                field_types: vec![DebugTypeId::new(0)],
            },
            EnumVariant {
                error: None,
                owner: EnumTypeId::new(0),
                name: StringId::new(0),
                fields: vec![StringId::new(0), StringId::new(0)],
                field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
            },
        ],
        debug_types: vec![
            crate::DebugType::Integer,
            crate::DebugType::String,
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
        distinct_types: Vec::new(),
    }
}

fn is_writable(value: &RetainedValue) -> bool {
    matches!(value.origin, super::MutationAccess::Writable(_))
}

#[test]
fn payload_children_are_writable_only_through_writable_typed_roots() {
    use crate::DebugTypeId;

    let limits = DebugInspectionLimits::default();
    let executable = payload_executable();
    let enumeration = Value::Enum(SharedEnum::new(
        Arc::new(RuntimeEnumLayout {
            error: None,
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
            value.origin = super::MutationAccess::NotMutable;
            value
        },
        limits,
        Some(&executable),
    );
    assert!(matches!(
        immutable.children[0].origin,
        super::MutationAccess::NotMutable
    ));
    assert_eq!(immutable.children[0].name, "value");
}
