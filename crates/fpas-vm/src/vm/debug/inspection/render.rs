//! Non-recursive bounded summaries and lazy aggregate child extraction.

use std::collections::HashSet;
use std::sync::{Arc, TryLockError};

use fpas_bytecode::Value;

use super::model::DebugInspectionLimits;

#[derive(Clone)]
pub(super) struct RetainedValue {
    pub name: String,
    pub value: Option<Value>,
    pub type_name: String,
    pub presentation_hint: Option<String>,
    pub depth: usize,
    pub visited_cells: HashSet<usize>,
}

pub(super) struct RenderedValue {
    pub summary: String,
    pub type_name: String,
    pub children: Vec<RetainedValue>,
    pub named_children: usize,
    pub indexed_children: usize,
    pub presentation_hint: Option<String>,
}

pub(super) fn render(value: &RetainedValue, limits: DebugInspectionLimits) -> RenderedValue {
    let Some(runtime) = &value.value else {
        return RenderedValue {
            summary: "<uninitialized>".to_string(),
            type_name: value.type_name.clone(),
            children: Vec::new(),
            named_children: 0,
            indexed_children: 0,
            presentation_hint: value.presentation_hint.clone(),
        };
    };
    if value.depth >= limits.max_depth {
        return leaf(
            "<max depth>".to_string(),
            runtime.type_name().to_string(),
            Some("truncated".to_string()),
        );
    }
    match runtime {
        Value::Integer(number) => leaf(number.to_string(), value.type_name.clone(), None),
        Value::Real(number) => leaf(real(*number), value.type_name.clone(), None),
        Value::Boolean(boolean) => leaf(boolean.to_string(), value.type_name.clone(), None),
        Value::Str(string) => leaf(
            quoted(string, limits.max_string_chars),
            value.type_name.clone(),
            (string.chars().count() > limits.max_string_chars).then(|| "truncated".to_string()),
        ),
        Value::Unit => leaf("()".to_string(), value.type_name.clone(), None),
        Value::OptionNone => leaf("None".to_string(), value.type_name.clone(), None),
        Value::Task(id) => leaf(format!("<task {id}>"), value.type_name.clone(), None),
        Value::OpaqueHandle(_) => leaf(
            "<opaque handle>".to_string(),
            value.type_name.clone(),
            Some("opaque".to_string()),
        ),
        Value::Array(values) => aggregate(
            format!("[{} items]", values.len()),
            value.type_name.clone(),
            values
                .iter()
                .enumerate()
                .map(|(index, child)| child_value(format!("[{index}]"), child, value))
                .collect(),
            0,
            values.len(),
            value.presentation_hint.clone(),
            limits,
        ),
        Value::Dict(values) => {
            let children = values
                .iter()
                .enumerate()
                .flat_map(|(index, (key, item))| {
                    [
                        child_value(format!("[{index}].key"), key, value),
                        child_value(format!("[{index}].value"), item, value),
                    ]
                })
                .collect();
            aggregate(
                format!("{{{} entries}}", values.len()),
                value.type_name.clone(),
                children,
                values.len().saturating_mul(2),
                0,
                value.presentation_hint.clone(),
                limits,
            )
        }
        Value::Record(record) => {
            let body = record.body();
            let children = body
                .layout
                .fields
                .iter()
                .zip(&body.values)
                .map(|(name, child)| child_value(name.clone(), child, value))
                .collect();
            aggregate(
                format!("{} {{...}}", body.layout.type_name),
                body.layout.type_name.clone(),
                children,
                body.values.len(),
                0,
                value.presentation_hint.clone(),
                limits,
            )
        }
        Value::Enum(enumeration) => {
            let body = enumeration.body();
            let children = body
                .layout
                .fields
                .iter()
                .zip(&body.values)
                .map(|(name, child)| child_value(name.clone(), child, value))
                .collect();
            aggregate(
                format!("{}.{}", body.layout.type_name, body.layout.variant),
                body.layout.type_name.clone(),
                children,
                body.values.len(),
                0,
                value.presentation_hint.clone(),
                limits,
            )
        }
        Value::ResultOk(inner) => wrapper("Ok", inner, value, limits),
        Value::ResultError(inner) => wrapper("Error", inner, value, limits),
        Value::OptionSome(inner) => wrapper("Some", inner, value, limits),
        Value::Function(function) => aggregate(
            format!("<function {}>", function.name),
            value.type_name.clone(),
            function
                .captures
                .iter()
                .enumerate()
                .map(|(index, child)| child_value(format!("capture[{index}]"), child, value))
                .collect(),
            function.captures.len(),
            0,
            value.presentation_hint.clone(),
            limits,
        ),
        Value::Cell(cell) => render_cell(cell, value, limits),
    }
}

fn render_cell(
    cell: &Arc<std::sync::Mutex<Value>>,
    parent: &RetainedValue,
    limits: DebugInspectionLimits,
) -> RenderedValue {
    let identity = Arc::as_ptr(cell) as usize;
    if parent.visited_cells.contains(&identity) {
        return leaf(
            "<cycle>".to_string(),
            parent.type_name.clone(),
            Some("cycle".to_string()),
        );
    }
    let (inner, poisoned) = match cell.try_lock() {
        Ok(inner) => (inner.clone(), false),
        Err(TryLockError::Poisoned(poisoned)) => (poisoned.into_inner().clone(), true),
        Err(TryLockError::WouldBlock) => {
            return leaf(
                "<cell busy>".to_string(),
                parent.type_name.clone(),
                Some("unavailable".to_string()),
            );
        }
    };
    let mut child = child_value("value".to_string(), &inner, parent);
    child.visited_cells.insert(identity);
    let visited_cells = child.visited_cells.clone();
    let mut rendered = aggregate(
        "<mutable cell>".to_string(),
        parent.type_name.clone(),
        vec![child],
        1,
        0,
        Some(if poisoned {
            "captured mutable, poisoned".to_string()
        } else {
            "captured mutable".to_string()
        }),
        limits,
    );
    if parent.presentation_hint.as_deref() == Some("captured mutable") {
        let inner_value = RetainedValue {
            name: parent.name.clone(),
            value: Some(inner),
            type_name: parent.type_name.clone(),
            presentation_hint: parent.presentation_hint.clone(),
            depth: parent.depth,
            visited_cells,
        };
        let transparent = render(&inner_value, limits);
        rendered.summary = transparent.summary;
    }
    rendered
}

fn wrapper(
    name: &str,
    inner: &Value,
    parent: &RetainedValue,
    limits: DebugInspectionLimits,
) -> RenderedValue {
    aggregate(
        format!("{name}(...)"),
        parent.type_name.clone(),
        vec![child_value("value".to_string(), inner, parent)],
        1,
        0,
        parent.presentation_hint.clone(),
        limits,
    )
}

fn child_value(name: String, value: &Value, parent: &RetainedValue) -> RetainedValue {
    RetainedValue {
        name,
        value: Some(value.clone()),
        type_name: value.type_name().to_string(),
        presentation_hint: None,
        depth: parent.depth.saturating_add(1),
        visited_cells: parent.visited_cells.clone(),
    }
}

fn aggregate(
    summary: String,
    type_name: String,
    mut children: Vec<RetainedValue>,
    named_children: usize,
    indexed_children: usize,
    presentation_hint: Option<String>,
    limits: DebugInspectionLimits,
) -> RenderedValue {
    let truncated = children.len() > limits.max_children;
    children.truncate(limits.max_children);
    RenderedValue {
        summary,
        type_name,
        children,
        named_children,
        indexed_children,
        presentation_hint: if truncated {
            Some(match presentation_hint {
                Some(hint) => format!("{hint}, truncated"),
                None => "truncated".to_string(),
            })
        } else {
            presentation_hint
        },
    }
}

fn leaf(summary: String, type_name: String, hint: Option<String>) -> RenderedValue {
    RenderedValue {
        summary,
        type_name,
        children: Vec::new(),
        named_children: 0,
        indexed_children: 0,
        presentation_hint: hint,
    }
}

fn real(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_string()
    } else if value == f64::INFINITY {
        "Infinity".to_string()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".to_string()
    } else {
        value.to_string()
    }
}

fn quoted(value: &str, maximum: usize) -> String {
    let mut output = String::from("'");
    for character in value.chars().take(maximum) {
        if character == '\'' {
            output.push_str("''");
        } else {
            output.push(character);
        }
    }
    if value.chars().count() > maximum {
        output.push('…');
    }
    output.push('\'');
    output
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use fpas_bytecode::{
        EnumTypeId, EnumVariantId, FunctionId, RecordTypeId, RuntimeEnumLayout,
        RuntimeRecordLayout, SharedArray, SharedEnum, SharedRecord, Value,
    };

    use super::*;

    fn retained(value: Value) -> RetainedValue {
        RetainedValue {
            name: "value".to_string(),
            type_name: value.type_name().to_string(),
            value: Some(value),
            presentation_hint: None,
            depth: 0,
            visited_cells: HashSet::new(),
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

        let function = Value::function(
            FunctionId::new(0),
            "callback".to_string(),
            vec![Value::Integer(1)],
            false,
        );
        let function = render(&retained(function), limits);
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
        let rendered = render(&retained(Value::Cell(Arc::clone(&busy))), limits);
        assert_eq!(rendered.summary, "<cell busy>");
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
        let rendered = render(&retained(Value::Cell(poisoned)), limits);
        assert_eq!(
            rendered.presentation_hint,
            Some("captured mutable, poisoned".to_string())
        );
    }
}
