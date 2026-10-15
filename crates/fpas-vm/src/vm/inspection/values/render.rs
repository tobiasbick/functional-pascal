//! Non-recursive bounded summaries and lazy aggregate child extraction.

use std::sync::{Arc, TryLockError};

use fpas_bytecode::{DebugType, Executable, Value};

use super::{ValueLimits, ValueOrigin};
use super::{ValuePath as MutationPath, payload::resolve as resolve_payload};

pub(in crate::vm) use super::model::{RenderedValue, RetainedValue};

/// Borrow a bounded metadata prefix before allocating names or constructor summaries.
pub(in crate::vm) fn metadata(text: &str) -> &str {
    let end = text
        .char_indices()
        .nth(4096)
        .map_or(text.len(), |(index, _)| index);
    &text[..end]
}

/// Present a value without invoking FPAS code or waiting for a cell lock.
pub(in crate::vm) fn render_with_executable<O: ValueOrigin>(
    value: &RetainedValue<O>,
    limits: ValueLimits,
    executable: Option<&Executable>,
) -> RenderedValue<O> {
    let Some(runtime) = &value.value else {
        return RenderedValue {
            summary: if value
                .presentation_hint
                .as_deref()
                .is_some_and(|hint| hint.starts_with("unavailable:"))
            {
                "<unavailable>"
            } else {
                "<uninitialized>"
            }
            .to_string(),
            type_name: metadata(&value.type_name).to_string(),
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
        Value::Integer(_)
        | Value::Real(_)
        | Value::Boolean(_)
        | Value::Str(_)
        | Value::Unit
        | Value::OptionNone
        | Value::Task(_)
        | Value::OpaqueHandle(_) => {
            let mut budget = super::budget::ValueBudget::new(1, usize::MAX);
            let owned = super::snapshot::capture(value, limits, executable, &mut budget);
            match owned {
                Some(owned) => leaf(
                    owned.summary,
                    metadata(&value.type_name).to_string(),
                    if matches!(runtime, Value::Str(text) if text.chars().nth(limits.max_string_chars).is_some())
                    {
                        Some("truncated".to_string())
                    } else if matches!(runtime, Value::OpaqueHandle(_)) {
                        Some("opaque".to_string())
                    } else {
                        None
                    },
                ),
                None => leaf(
                    "<unavailable>".to_string(),
                    metadata(&value.type_name).to_string(),
                    Some("unavailable".to_string()),
                ),
            }
        }
        Value::Array(values) => aggregate(
            format!("[{} items]", values.len()),
            metadata(&value.type_name).to_string(),
            values
                .iter()
                .take(limits.max_children)
                .enumerate()
                .map(|(index, child)| {
                    let ty = child_type(value, executable, |ty| match ty {
                        DebugType::Array(inner) => Some(*inner),
                        _ => None,
                    });
                    child_value(
                        format!("[{index}]"),
                        child,
                        value,
                        ty.map(|ty| (MutationPath::ArrayIndex(index), ty)),
                    )
                })
                .collect(),
            0,
            values.len(),
            value.presentation_hint.clone(),
            limits,
        ),
        Value::Dict(values) => {
            let children = values
                .iter()
                .take(limits.max_children / 2)
                .enumerate()
                .flat_map(|(index, (key, item))| {
                    [
                        child_value(format!("[{index}].key"), key, value, None),
                        child_value(
                            format!("[{index}].value"),
                            item,
                            value,
                            child_type(value, executable, |ty| match ty {
                                DebugType::Dictionary { value, .. } => Some(*value),
                                _ => None,
                            })
                            .map(|ty| (MutationPath::DictionaryValue(key.clone()), ty)),
                        ),
                    ]
                })
                .collect();
            aggregate(
                format!("{{{} entries}}", values.len()),
                metadata(&value.type_name).to_string(),
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
                .take(limits.max_children)
                .enumerate()
                .map(|(index, (name, child))| {
                    let ty = executable.and_then(|executable| {
                        executable
                            .records
                            .get(usize::from(body.layout.record.get()))
                            .and_then(|layout| layout.fields.get(index))
                            .map(|field| field.ty)
                    });
                    let mut retained = child_value(
                        metadata(name).to_string(),
                        child,
                        value,
                        ty.map(|ty| (MutationPath::RecordField(index), ty)),
                    );
                    if metadata(name).len() < name.len() {
                        retained.presentation_hint = Some("truncated name".to_string());
                    }
                    retained
                })
                .collect();
            aggregate(
                format!("{} {{...}}", metadata(&body.layout.type_name)),
                metadata(&body.layout.type_name).to_string(),
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
                .take(limits.max_children)
                .map(|(name, child)| {
                    let mut retained = child_value(
                        metadata(name).to_string(),
                        child,
                        value,
                        payload_child(value, executable, runtime, name),
                    );
                    if metadata(name).len() < name.len() {
                        retained.presentation_hint = Some("truncated name".to_string());
                    }
                    retained
                })
                .collect();
            aggregate(
                format!(
                    "{}.{}",
                    metadata(&body.layout.type_name),
                    metadata(&body.layout.variant)
                ),
                metadata(&body.layout.type_name).to_string(),
                children,
                body.values.len(),
                0,
                value.presentation_hint.clone(),
                limits,
            )
        }
        Value::ResultOk(inner) => wrapper("Ok", inner, value, limits, executable, runtime),
        Value::ResultError(inner) => wrapper("Error", inner, value, limits, executable, runtime),
        Value::OptionSome(inner) => wrapper("Some", inner, value, limits, executable, runtime),
        Value::Function(function) => aggregate(
            format!("<function {}>", metadata(&function.name)),
            metadata(&value.type_name).to_string(),
            function
                .bound_receiver
                .iter()
                .map(|receiver| child_value("receiver".to_string(), receiver, value, None))
                .chain(function.captures.iter().enumerate().map(|(index, child)| {
                    child_value(format!("capture[{index}]"), child, value, None)
                }))
                .take(limits.max_children)
                .collect(),
            function
                .captures
                .len()
                .saturating_add(usize::from(function.bound_receiver.is_some())),
            0,
            value.presentation_hint.clone(),
            limits,
        ),
        Value::Cell(cell) => render_cell(cell, value, limits, executable),
        Value::Reference(_) => leaf(
            "<reference>".to_string(),
            metadata(&value.type_name).to_string(),
            Some("var parameter".to_string()),
        ),
    }
}

fn render_cell<O: ValueOrigin>(
    cell: &Arc<std::sync::Mutex<Value>>,
    parent: &RetainedValue<O>,
    limits: ValueLimits,
    executable: Option<&Executable>,
) -> RenderedValue<O> {
    let identity = Arc::as_ptr(cell) as usize;
    if parent.visited_cells.len() >= limits.max_depth {
        return leaf(
            "<max depth>".to_string(),
            metadata(&parent.type_name).to_string(),
            Some("truncated".to_string()),
        );
    }
    if parent.visited_cells.contains(&identity) {
        return leaf(
            "<cycle>".to_string(),
            metadata(&parent.type_name).to_string(),
            Some("cycle".to_string()),
        );
    }
    let (inner, poisoned) = match cell.try_lock() {
        Ok(inner) => (inner.clone(), false),
        Err(TryLockError::Poisoned(poisoned)) => (poisoned.into_inner().clone(), true),
        Err(TryLockError::WouldBlock) => {
            return leaf(
                "<cell busy>".to_string(),
                metadata(&parent.type_name).to_string(),
                Some("unavailable".to_string()),
            );
        }
    };
    let mut visited_cells = parent.visited_cells.clone();
    visited_cells.insert(identity);
    let children = if limits.max_children == 0 {
        Vec::new()
    } else {
        let mut child = child_value("value".to_string(), &inner, parent, None);
        child.origin = parent.origin.clone();
        child.visited_cells = visited_cells.clone();
        vec![child]
    };
    let mut rendered = aggregate(
        "<mutable cell>".to_string(),
        metadata(&parent.type_name).to_string(),
        children,
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
            name: metadata(&parent.name).to_string(),
            value: Some(inner),
            type_name: metadata(&parent.type_name).to_string(),
            presentation_hint: parent.presentation_hint.clone(),
            depth: parent.depth,
            visited_cells,
            debug_type: parent.debug_type,
            origin: parent.origin.clone(),
        };
        let transparent = render_with_executable(&inner_value, limits, executable);
        rendered.summary = transparent.summary;
    }
    rendered
}

fn wrapper<O: ValueOrigin>(
    name: &str,
    inner: &Value,
    parent: &RetainedValue<O>,
    limits: ValueLimits,
    executable: Option<&Executable>,
    runtime: &Value,
) -> RenderedValue<O> {
    aggregate(
        format!("{name}(...)"),
        metadata(&parent.type_name).to_string(),
        if limits.max_children == 0 {
            Vec::new()
        } else {
            vec![child_value(
                "value".to_string(),
                inner,
                parent,
                payload_child(parent, executable, runtime, "value"),
            )]
        },
        1,
        0,
        parent.presentation_hint.clone(),
        limits,
    )
}

fn payload_child<O: ValueOrigin>(
    parent: &RetainedValue<O>,
    executable: Option<&Executable>,
    runtime: &Value,
    name: &str,
) -> Option<(MutationPath, fpas_bytecode::DebugTypeId)> {
    let ty = parent.debug_type?;
    resolve_payload(executable?, ty, runtime, name).ok()
}

fn child_value<O: ValueOrigin>(
    name: String,
    value: &Value,
    parent: &RetainedValue<O>,
    writable: Option<(MutationPath, fpas_bytecode::DebugTypeId)>,
) -> RetainedValue<O> {
    RetainedValue {
        name,
        value: Some(value.clone()),
        type_name: value.type_name().to_string(),
        presentation_hint: None,
        depth: parent.depth.saturating_add(1),
        visited_cells: parent.visited_cells.clone(),
        debug_type: writable.as_ref().map(|(_, ty)| *ty),
        origin: writable.map_or(O::unsupported(), |(component, ty)| {
            parent.origin.descendant(component, ty)
        }),
    }
}

fn child_type<O: ValueOrigin>(
    value: &RetainedValue<O>,
    executable: Option<&Executable>,
    select: impl FnOnce(&DebugType) -> Option<fpas_bytecode::DebugTypeId>,
) -> Option<fpas_bytecode::DebugTypeId> {
    let ty = value.debug_type?;
    executable?
        .debug_types
        .get(ty.get() as usize)
        .and_then(select)
}

fn aggregate<O: ValueOrigin>(
    summary: String,
    type_name: String,
    mut children: Vec<RetainedValue<O>>,
    named_children: usize,
    indexed_children: usize,
    presentation_hint: Option<String>,
    limits: ValueLimits,
) -> RenderedValue<O> {
    let truncated = named_children.saturating_add(indexed_children) > children.len();
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

fn leaf<O: ValueOrigin>(
    summary: String,
    type_name: String,
    hint: Option<String>,
) -> RenderedValue<O> {
    RenderedValue {
        summary,
        type_name,
        children: Vec::new(),
        named_children: 0,
        indexed_children: 0,
        presentation_hint: hint,
    }
}
