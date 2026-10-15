//! Owned snapshot contracts beyond lazy debugger presentation.

use super::{
    ValueLimits,
    budget::ValueBudget,
    render::RetainedValue,
    snapshot::{self, Scalar, ValueSnapshot},
};
use fpas_bytecode::{FunctionId, Value};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

fn retained(value: Value) -> RetainedValue<()> {
    RetainedValue {
        name: "value".to_string(),
        type_name: value.type_name().to_string(),
        value: Some(value),
        presentation_hint: None,
        depth: 0,
        visited_cells: HashSet::new(),
        debug_type: None,
        origin: (),
    }
}

fn limits() -> ValueLimits {
    ValueLimits {
        max_children: 256,
        max_string_chars: 4096,
        max_summary_chars: 4096,
        max_depth: 16,
    }
}

fn capture(value: &Value, limits: ValueLimits, budget: &mut ValueBudget) -> ValueSnapshot {
    snapshot::capture(&retained(value.clone()), limits, None, budget).expect("root snapshot")
}

fn budget() -> ValueBudget {
    ValueBudget::new(16384, 8 * 1024 * 1024)
}

#[test]
fn snapshots_detach_nested_capture_cells_and_preserve_scalar_precision() {
    let minimum = capture(&Value::Integer(i64::MIN), limits(), &mut budget());
    assert_eq!(
        minimum.scalar,
        Some(Scalar::Integer("-9223372036854775808".to_string()))
    );
    let cell = Arc::new(Mutex::new(Value::Integer(i64::MAX)));
    let function = Value::function(
        FunctionId::new(0),
        "captured",
        vec![Value::Array(vec![Value::Cell(Arc::clone(&cell))].into())],
    );
    let snapshot = capture(&function, limits(), &mut budget());
    *cell.lock().expect("mutate live cell") = Value::Integer(0);
    assert_eq!(snapshot.kind, "function");
    assert_eq!(snapshot.summary, "<function captured>");
    let captured = &snapshot.children[0].1.children[0].1;
    assert_eq!(captured.kind, "integer");
    assert_eq!(
        captured.scalar,
        Some(Scalar::Integer("9223372036854775807".to_string()))
    );
    assert_eq!(
        Arc::strong_count(&cell),
        2,
        "only the test and the live function retain mutable storage"
    );
}

#[test]
fn snapshots_distinguish_busy_and_cyclic_cells_without_waiting() {
    let cell = Arc::new(Mutex::new(Value::Integer(1)));
    let guard = cell.lock().expect("hold cell");
    let busy = capture(
        &Value::Cell(Arc::clone(&cell)),
        ValueLimits {
            max_string_chars: 1,
            ..limits()
        },
        &mut budget(),
    );
    assert_eq!(
        busy.type_name, "c",
        "unavailable markers obey string limits too"
    );
    assert_eq!((busy.status, busy.reason), ("unavailable", Some("busy")));
    drop(guard);
    *cell.lock().expect("create cycle") = Value::Cell(Arc::clone(&cell));
    let cyclic = capture(&Value::Cell(Arc::clone(&cell)), limits(), &mut budget());
    assert_eq!(
        (cyclic.status, cyclic.reason),
        ("unavailable", Some("cycle"))
    );
    *cell.lock().expect("break cycle") = Value::Unit;
}

#[test]
fn snapshot_limits_bound_strings_depth_children_and_shared_storage() {
    let value = Value::dict(vec![
        (Value::Integer(1), Value::Str("a🙂'z".into())),
        (Value::Integer(2), Value::Unit),
    ]);
    let snapshot = capture(
        &value,
        ValueLimits {
            max_children: 3,
            ..limits()
        },
        &mut budget(),
    );
    assert_eq!(snapshot.total_children, Some(4));
    assert_eq!(snapshot.children.len(), 2, "dictionary pairs remain whole");
    assert_eq!(snapshot.reason, Some("child_limit"));
    assert_eq!(
        snapshot
            .children
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["[0].key", "[0].value"]
    );
    let short_text = capture(
        &Value::Str("a🙂'z".into()),
        ValueLimits {
            max_string_chars: 3,
            ..limits()
        },
        &mut budget(),
    );
    assert_eq!(short_text.scalar, Some(Scalar::String("a🙂'".to_string())));
    assert_eq!(short_text.summary, "'a🙂''…'");
    assert_eq!(short_text.reason, Some("string_limit"));
    let quoted = capture(
        &Value::Str("'".repeat(4096).into()),
        limits(),
        &mut budget(),
    );
    assert_eq!(
        quoted.summary.chars().count(),
        4096,
        "the summary has its own character bound including quoting"
    );
    assert_eq!(quoted.reason, Some("string_limit"));

    let nested = Value::Array(vec![Value::Array(vec![Value::Integer(5)].into())].into());
    let deep = capture(
        &nested,
        ValueLimits {
            max_depth: 1,
            ..limits()
        },
        &mut budget(),
    );
    assert_eq!(deep.children[0].1.reason, Some("depth_limit"));
    assert!(deep.children[0].1.children.is_empty());

    let mut shared = ValueBudget::new(2, 8 * 1024 * 1024);
    let limited = capture(
        &Value::Array(vec![Value::Integer(1); 20].into()),
        limits(),
        &mut shared,
    );
    assert_eq!(limited.children.len(), 1);
    assert_eq!(limited.reason, Some("value_limit"));
    assert!(snapshot::capture(&retained(Value::Unit), limits(), None, &mut shared).is_none());
    assert_eq!(shared.exhausted, Some("value_limit"));
    let pair = capture(&value, limits(), &mut ValueBudget::new(2, 8 * 1024 * 1024));
    assert!(pair.children.is_empty());
    assert_eq!(
        pair.reason,
        Some("value_limit"),
        "a partial dictionary pair cannot bypass the value budget"
    );

    let mut bytes = ValueBudget::new(16384, std::mem::size_of::<ValueSnapshot>() + 12);
    let short = capture(
        &Value::Str("🙂".repeat(100000).into()),
        limits(),
        &mut bytes,
    );
    assert_eq!(short.reason, Some("retained_byte_limit"));
    assert!(bytes.bytes < 4, "UTF-8 scalars are retained whole");
    assert!(snapshot::capture(&retained(Value::Unit), limits(), None, &mut bytes).is_none());
    let tight = capture(
        &nested,
        limits(),
        &mut ValueBudget::new(16384, std::mem::size_of::<ValueSnapshot>() + 100),
    );
    assert_eq!(tight.reason, Some("retained_byte_limit"));
    let named = capture(
        &Value::function(FunctionId::new(0), "x".repeat(100000), Vec::new()),
        limits(),
        &mut budget(),
    );
    assert!(named.summary.len() <= 4096);
    assert_eq!(named.reason, Some("string_limit"));
}

#[test]
fn missing_field_metadata_is_unavailable_instead_of_inventing_names() {
    let value = Value::Record(fpas_bytecode::SharedRecord::new(
        Arc::new(fpas_bytecode::RuntimeRecordLayout {
            record: fpas_bytecode::RecordTypeId::new(0),
            type_name: "Missing".to_string(),
            fields: Vec::new(),
        }),
        vec![Value::Integer(1)],
    ));
    let snapshot = capture(&value, limits(), &mut budget());
    assert_eq!(
        (snapshot.status, snapshot.reason),
        ("unavailable", Some("missing_metadata"))
    );
    assert_eq!(snapshot.total_children, Some(1));
    assert!(snapshot.children.is_empty());
}

#[test]
fn aggregate_limits_apply_before_child_origins_are_materialized() {
    use super::{ValueOrigin, ValuePath};
    use fpas_bytecode::{DebugType, DebugTypeId};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    struct Count(Option<Arc<AtomicUsize>>);
    impl ValueOrigin for Count {
        fn unsupported() -> Self {
            Self(None)
        }
        fn descendant(&self, _: ValuePath, _: DebugTypeId) -> Self {
            self.0
                .as_ref()
                .expect("parent origin")
                .fetch_add(1, Ordering::Relaxed);
            self.clone()
        }
    }
    let (program, errors) = fpas_parser::parse("program Bounds; begin end.");
    assert!(errors.is_empty());
    let executable = fpas_compiler::compile(&program).expect("metadata image");
    let mut image = executable.executable().clone();
    image.debug_types = vec![DebugType::Integer, DebugType::Array(DebugTypeId::new(0))];
    let materialized = Arc::new(AtomicUsize::new(0));
    let input = RetainedValue {
        name: String::new(),
        value: Some(Value::Array(vec![Value::Integer(1); 10000].into())),
        type_name: "array of Integer".to_string(),
        presentation_hint: None,
        depth: 0,
        visited_cells: HashSet::new(),
        debug_type: Some(DebugTypeId::new(1)),
        origin: Count(Some(Arc::clone(&materialized))),
    };
    let rendered = super::render::render_with_executable(
        &input,
        ValueLimits {
            max_children: 2,
            ..limits()
        },
        Some(&image),
    );
    assert_eq!(rendered.indexed_children, 10000);
    assert_eq!(rendered.children.len(), 2);
    assert_eq!(
        materialized.load(Ordering::Relaxed),
        2,
        "omitted children must never allocate origins"
    );
}
