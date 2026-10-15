//! Detached value trees: no mutable cells, runtime handles, or debugger origins.

use super::{
    ValueLimits, ValueOrigin,
    budget::ValueBudget,
    render::{self, RetainedValue},
};
use fpas_bytecode::{Executable, Value};
use std::sync::{Arc, TryLockError};

/// Canonical scalar payload, preserving the full FPAS integer range.
#[derive(Debug, PartialEq)]
pub(in crate::vm) enum Scalar {
    /// Integers are decimal strings, never lossy JSON numbers.
    Integer(String),
    /// Finite binary64 value.
    Real(f64),
    /// Boolean value.
    Boolean(bool),
    /// Owned UTF-8 text.
    String(String),
    /// Runtime task identity without a native handle.
    Task(u64),
}

/// Immutable bounded value data suitable for retaining after a worker is dropped.
#[derive(Debug, PartialEq)]
pub(in crate::vm) struct ValueSnapshot {
    /// Source or runtime type name.
    pub type_name: String,
    /// Stable value category.
    pub kind: &'static str,
    /// Available, uninitialized, unavailable, or truncated.
    pub status: &'static str,
    /// Scalar content when applicable.
    pub scalar: Option<Scalar>,
    /// Bounded non-executing display summary.
    pub summary: String,
    /// Children in source/runtime order.
    pub children: Vec<(String, ValueSnapshot)>,
    /// Full immediate child count, when known.
    pub total_children: Option<usize>,
    /// Stable reason for unavailable or truncated data.
    pub reason: Option<&'static str>,
}

/// Capture an owned tree, charging a shared budget before retaining its contents.
/// `None` means even a root marker does not fit; the containing capture records the omission.
pub(in crate::vm) fn capture<O: ValueOrigin>(
    value: &RetainedValue<O>,
    limits: ValueLimits,
    executable: Option<&Executable>,
    budget: &mut ValueBudget,
) -> Option<ValueSnapshot> {
    budget.exhausted = None;
    if !budget.node(std::mem::size_of::<ValueSnapshot>()) {
        return None;
    }
    capture_reserved(value, limits, executable, budget)
}

fn capture_reserved<O: ValueOrigin>(
    value: &RetainedValue<O>,
    limits: ValueLimits,
    executable: Option<&Executable>,
    budget: &mut ValueBudget,
) -> Option<ValueSnapshot> {
    if let Some(Value::Cell(cell)) = &value.value {
        let identity = Arc::as_ptr(cell) as usize;
        let reason = if value.visited_cells.contains(&identity) {
            Some("cycle")
        } else if value.visited_cells.len() >= limits.max_depth {
            Some("depth_limit")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Some(marker(value, reason, limits.max_string_chars, budget));
        }
        let inner = match cell.try_lock() {
            Ok(inner) => inner.clone(),
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner().clone(),
            Err(TryLockError::WouldBlock) => {
                return Some(marker(value, "busy", limits.max_string_chars, budget));
            }
        };
        let mut inner_value = RetainedValue {
            name: String::new(),
            type_name: if value.type_name == "cell" {
                inner.type_name()
            } else {
                render::metadata(&value.type_name)
            }
            .to_string(),
            value: Some(inner),
            presentation_hint: None,
            depth: value.depth,
            visited_cells: value.visited_cells.clone(),
            debug_type: value.debug_type,
            origin: value.origin.clone(),
        };
        inner_value.visited_cells.insert(identity);
        let mut node = capture_reserved(&inner_value, limits, executable, budget)?;
        if render::metadata(&value.type_name).len() < value.type_name.len() {
            node.status = "truncated";
            node.reason = Some("string_limit");
        }
        return Some(node);
    }
    if let Some(node) = super::atomic::capture(value, limits, budget) {
        return Some(node);
    }
    let storage_children = budget.bytes / std::mem::size_of::<ValueSnapshot>();
    let budget_limit = if budget.values < limits.max_children && budget.values <= storage_children {
        Some("value_limit")
    } else if storage_children < limits.max_children {
        Some("retained_byte_limit")
    } else {
        None
    };
    let child_limit = if value.depth >= limits.max_depth {
        0
    } else {
        limits.max_children.min(budget.values).min(storage_children)
    };
    let view = render::render_with_executable(
        value,
        ValueLimits {
            max_children: child_limit,
            max_depth: usize::MAX,
            ..limits
        },
        executable,
    );
    let (type_name, type_truncated) = budget.text(&view.type_name, limits.max_string_chars);
    let (summary, summary_truncated) = budget.text(&view.summary, limits.max_summary_chars);
    let runtime = value.value.as_ref();
    let mut node = ValueSnapshot {
        type_name,
        kind: runtime.map_or("unknown", kind),
        status: "available",
        scalar: None,
        summary,
        children: Vec::new(),
        total_children: runtime.map(|_| view.named_children.saturating_add(view.indexed_children)),
        reason: None,
    };
    if type_truncated || summary_truncated {
        node.status = "truncated";
        node.reason = Some(budget.exhausted.unwrap_or("string_limit"));
    }
    let metadata_limited = value.type_name.chars().nth(4096).is_some()
        || match runtime {
            Some(Value::Record(record)) => {
                record.body().layout.type_name.chars().nth(4096).is_some()
            }
            Some(Value::Enum(enumeration)) => {
                enumeration
                    .body()
                    .layout
                    .type_name
                    .chars()
                    .nth(4096)
                    .is_some()
                    || enumeration
                        .body()
                        .layout
                        .variant
                        .chars()
                        .nth(4096)
                        .is_some()
            }
            Some(Value::Function(function)) => function.name.chars().nth(4096).is_some(),
            _ => false,
        };
    if metadata_limited {
        node.status = "truncated";
        node.reason = Some("string_limit");
    }
    let missing_metadata = match runtime {
        Some(Value::Record(record)) => {
            record.body().layout.fields.len() != record.body().values.len()
        }
        Some(Value::Enum(enumeration)) => {
            enumeration.body().layout.fields.len() != enumeration.body().values.len()
        }
        _ => false,
    };
    for child in &view.children {
        let entry_size = std::mem::size_of::<(String, ValueSnapshot)>();
        if !budget.charge(entry_size) {
            break;
        }
        let (name, truncated) = budget.text(&child.name, limits.max_string_chars);
        let Some(snapshot) = capture(child, limits, executable, budget) else {
            break;
        };
        node.children.push((name, snapshot));
        if truncated {
            node.status = "truncated";
            node.reason = Some(budget.exhausted.unwrap_or("string_limit"));
        }
        if child.presentation_hint.as_deref() == Some("truncated name") {
            node.status = "truncated";
            node.reason = Some("string_limit");
        }
    }
    if node.kind == "dict" && !node.children.len().is_multiple_of(2) {
        node.children.pop();
    }
    if node
        .total_children
        .is_some_and(|total| total > node.children.len())
    {
        node.status = "truncated";
        node.reason = Some(
            budget
                .exhausted
                .unwrap_or(if value.depth >= limits.max_depth {
                    "depth_limit"
                } else {
                    budget_limit.unwrap_or("child_limit")
                }),
        );
    }
    if missing_metadata {
        node.status = "unavailable";
        node.reason = Some("missing_metadata");
    }
    Some(node)
}

fn marker<O: ValueOrigin>(
    value: &RetainedValue<O>,
    reason: &'static str,
    maximum_chars: usize,
    budget: &mut ValueBudget,
) -> ValueSnapshot {
    let (type_name, _) = budget.text(&value.type_name, maximum_chars);
    ValueSnapshot {
        type_name,
        kind: "unknown",
        status: if reason == "depth_limit" {
            "truncated"
        } else {
            "unavailable"
        },
        scalar: None,
        summary: String::new(),
        children: Vec::new(),
        total_children: None,
        reason: Some(reason),
    }
}

/// Stable value category without running user code.
pub(super) fn kind(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "integer",
        Value::Real(_) => "real",
        Value::Boolean(_) => "boolean",
        Value::Str(_) => "string",
        Value::Unit => "unit",
        Value::Record(_) => "record",
        Value::Enum(_) => "enum",
        Value::Array(_) => "array",
        Value::Dict(_) => "dict",
        Value::OptionNone | Value::OptionSome(_) => "option",
        Value::ResultOk(_) | Value::ResultError(_) => "result",
        Value::Function(_) => "function",
        Value::Task(_) => "task",
        Value::OpaqueHandle(_) => "opaque",
        Value::Cell(_) | Value::Reference(_) => "unknown",
    }
}
