//! Canonical scalar payloads and summaries shared by snapshots and lazy presentation.

use super::{
    ValueLimits, ValueOrigin,
    budget::ValueBudget,
    render::RetainedValue,
    snapshot::{Scalar, ValueSnapshot},
};
use fpas_bytecode::Value;

/// Capture a leaf after its node has been reserved; aggregates return `None`.
pub(super) fn capture<O: ValueOrigin>(
    value: &RetainedValue<O>,
    limits: ValueLimits,
    budget: &mut ValueBudget,
) -> Option<ValueSnapshot> {
    let runtime = value.value.as_ref();
    if matches!(
        runtime,
        Some(
            Value::Array(_)
                | Value::Dict(_)
                | Value::Record(_)
                | Value::Enum(_)
                | Value::Function(_)
                | Value::ResultOk(_)
                | Value::ResultError(_)
                | Value::OptionSome(_)
                | Value::Cell(_)
        )
    ) {
        return None;
    }
    let (type_name, type_truncated) = budget.text(&value.type_name, limits.max_string_chars);
    let mut node = ValueSnapshot {
        type_name,
        kind: runtime.map_or("unknown", super::snapshot::kind),
        status: "available",
        scalar: None,
        summary: String::new(),
        children: Vec::new(),
        total_children: runtime.map(|_| 0),
        reason: None,
    };
    let mut truncated = type_truncated;
    let summary = match runtime {
        Some(Value::Integer(number)) => {
            let text = number.to_string();
            let (payload, limited) = budget.text(&text, 20);
            truncated |= limited;
            node.scalar = (!limited).then_some(Scalar::Integer(payload));
            text
        }
        Some(Value::Real(number)) => {
            node.scalar = number.is_finite().then_some(Scalar::Real(*number));
            if number.is_nan() {
                "NaN".to_string()
            } else if *number == f64::INFINITY {
                "Infinity".to_string()
            } else if *number == f64::NEG_INFINITY {
                "-Infinity".to_string()
            } else {
                number.to_string()
            }
        }
        Some(Value::Boolean(boolean)) => {
            node.scalar = Some(Scalar::Boolean(*boolean));
            boolean.to_string()
        }
        Some(Value::Str(text)) => {
            let (payload, limited) = budget.text(text, limits.max_string_chars);
            truncated |= limited;
            let mut summary = String::from("'");
            // Each summary byte is charged separately from the scalar payload.
            for character in payload.chars() {
                if character == '\'' {
                    summary.push('\'');
                }
                summary.push(character);
            }
            if limited {
                summary.push('…');
            }
            summary.push('\'');
            node.scalar = Some(Scalar::String(payload));
            summary
        }
        Some(Value::Unit) => "()".to_string(),
        Some(Value::OptionNone) => "None".to_string(),
        Some(Value::Task(id)) => {
            node.scalar = Some(Scalar::Task(*id));
            format!("<task {id}>")
        }
        Some(Value::OpaqueHandle(_)) => "<opaque handle>".to_string(),
        Some(Value::Reference(_)) => {
            node.status = "unavailable";
            node.reason = Some("storage_unavailable");
            "<reference>".to_string()
        }
        None => {
            let unavailable = value
                .presentation_hint
                .as_deref()
                .and_then(|hint| hint.strip_prefix("unavailable: "));
            node.status = if unavailable.is_some() {
                "unavailable"
            } else {
                "uninitialized"
            };
            node.reason = unavailable.map(|reason| match reason {
                "busy" => "busy",
                "missing_metadata" => "missing_metadata",
                _ => "storage_unavailable",
            });
            if unavailable.is_some() {
                "<unavailable>".to_string()
            } else {
                "<uninitialized>".to_string()
            }
        }
        _ => return None,
    };
    let (summary, limited) = budget.text(&summary, limits.max_summary_chars);
    node.summary = summary;
    if truncated || limited {
        node.status = "truncated";
        node.reason = Some(budget.exhausted.unwrap_or("string_limit"));
    }
    Some(node)
}
