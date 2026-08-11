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
        Value::ResultOk(Box::new(Value::Integer(1))),
        Value::ResultError(Box::new(Value::Str("error".into()))),
        Value::OptionSome(Box::new(Value::Boolean(true))),
    ] {
        assert_eq!(render(&retained(wrapper), limits).named_children, 1);
    }
    assert_eq!(render(&retained(Value::OptionNone), limits).summary, "None");

    let function = render(
        &retained(Value::function(
            FunctionId::new(0),
            "callback".to_string(),
            vec![Value::Integer(1)],
            false,
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
