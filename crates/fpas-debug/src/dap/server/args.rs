//! DAP argument extraction onto typed debug engine operations.

use serde_json::Value;

use crate::engine::{AssignOp, DataBreakpointOp, FunctionBreakpointOp};

/// Required DAP string field.
pub(super) fn required_string(arguments: &Value, field: &str) -> Result<String, String> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("DAP request requires `{field}`."))
}

/// Required DAP unsigned integer field.
pub(super) fn required_u64(arguments: &Value, field: &str) -> Result<u64, String> {
    arguments
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("DAP request requires `{field}`."))
}

/// Optional DAP unsigned integer; missing or null is `None`.
pub(super) fn optional_u64(arguments: &Value, field: &str) -> Result<Option<u64>, String> {
    match arguments.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("DAP request `{field}` must be a non-negative integer.")),
    }
}

/// Optional DAP string; missing or null is `None`.
pub(super) fn optional_string(arguments: &Value, field: &str) -> Option<String> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Optional expression string; rejects non-string values when present.
pub(super) fn optional_expression(
    arguments: &Value,
    field: &str,
) -> Result<Option<String>, String> {
    match arguments.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(source)) => Ok(Some(source.clone())),
        Some(_) => Err(format!(
            "DAP request `{field}` must be a string when present."
        )),
    }
}

/// Pagination count: missing or zero means the advertised default.
pub(super) fn page_count(value: Option<&Value>, all_count: usize) -> usize {
    match value.and_then(Value::as_u64) {
        None | Some(0) => all_count,
        Some(count) => usize::try_from(count).unwrap_or(all_count),
    }
}

/// Parse DAP `assign` onto an engine assign payload.
pub(super) fn parse_assign(value: Option<&Value>) -> Result<Option<AssignOp>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(object) = value.as_object() else {
        return Err("DAP assign must be an object with `identity` and `expression`.".to_string());
    };
    let Some(identity) = object.get("identity").and_then(parse_identity) else {
        return Err("DAP assign requires a location identity.".to_string());
    };
    let Some(expression) = object.get("expression").and_then(Value::as_str) else {
        return Err("DAP assign requires string field `expression`.".to_string());
    };
    Ok(Some(AssignOp {
        identity,
        expression: expression.to_string(),
    }))
}

/// Parse a DAP or JSONL-shaped location identity.
pub(super) fn parse_identity(value: &Value) -> Option<fpas_vm::DebugDataLocationIdentity> {
    let object = value.as_object()?;
    if let Some(index) = object.get("index").and_then(Value::as_u64) {
        return Some(fpas_vm::DebugDataLocationIdentity::Global { index });
    }
    Some(fpas_vm::DebugDataLocationIdentity::FrameRegister {
        task_id: object
            .get("taskId")
            .or_else(|| object.get("task_id"))
            .and_then(Value::as_u64)?,
        function: object.get("function").and_then(Value::as_u64)?,
        register: object.get("register").and_then(Value::as_u64)?,
    })
}

/// Parse DAP function-breakpoint descriptors.
pub(super) fn parse_function_breakpoints(
    arguments: &Value,
) -> Result<Vec<FunctionBreakpointOp>, String> {
    let requested = arguments
        .get("breakpoints")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    requested
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                return Err(format!(
                    "Function breakpoint at index {index} requires string field `name`."
                ));
            };
            Ok(FunctionBreakpointOp {
                name: name.to_string(),
                condition: optional_string(item, "condition"),
                hit_condition: optional_string(item, "hitCondition"),
            })
        })
        .collect()
}

/// Parse DAP data-breakpoint descriptors from `dataId` values.
pub(super) fn parse_data_breakpoints(arguments: &Value) -> Result<Vec<DataBreakpointOp>, String> {
    let requested = arguments
        .get("breakpoints")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut breakpoints = Vec::with_capacity(requested.len());
    for breakpoint in requested {
        let Some(data_id) = breakpoint.get("dataId").and_then(Value::as_str) else {
            return Err("setDataBreakpoints requires dataId from dataBreakpointInfo.".to_string());
        };
        let Some(identity) = identity_from_data_id(data_id) else {
            return Err(
                "setDataBreakpoints dataId must name a global from dataBreakpointInfo.".to_string(),
            );
        };
        let access = match breakpoint.get("accessType").and_then(Value::as_str) {
            None | Some("write") => fpas_vm::DataBreakpointAccess::Write,
            Some("change") => fpas_vm::DataBreakpointAccess::Change,
            Some("read" | "readWrite") => fpas_vm::DataBreakpointAccess::Read,
            Some(other) => {
                return Err(format!("Unsupported data-breakpoint accessType `{other}`."));
            }
        };
        breakpoints.push(DataBreakpointOp {
            identity,
            access,
            assign: parse_assign(breakpoint.get("assign"))?,
        });
    }
    Ok(breakpoints)
}

/// Parse DAP exception-filter names.
pub(super) fn parse_filters(arguments: &Value) -> Result<Vec<String>, String> {
    let Some(requested) = arguments.get("filters").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    requested
        .iter()
        .enumerate()
        .map(|(index, filter)| {
            filter
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("Runtime failure filter at index {index} must be a string."))
        })
        .collect()
}

/// Parse DAP variant construction field map.
pub(super) fn parse_variant_fields(arguments: &Value) -> Result<Vec<(String, String)>, String> {
    match arguments.get("fields") {
        None | Some(Value::Null) => Err("DAP request requires `fields`.".to_string()),
        Some(Value::Object(fields)) => fields
            .iter()
            .map(|(name, value)| {
                let Some(source) = value.as_str() else {
                    return Err(format!(
                        "DAP field `{name}` must be an FPAS expression string."
                    ));
                };
                Ok((name.clone(), source.to_string()))
            })
            .collect(),
        Some(_) => Err("DAP request `fields` must be an object.".to_string()),
    }
}

/// Reject DAP arguments other than the listed field names.
pub(super) fn reject_unknown_fields(arguments: &Value, allowed: &[&str]) -> Result<(), String> {
    let Some(object) = arguments.as_object() else {
        return Ok(());
    };
    match object.keys().find(|key| !allowed.contains(&key.as_str())) {
        Some(name) => Err(format!("DAP request does not accept `{name}`.")),
        None => Ok(()),
    }
}

fn identity_from_data_id(data_id: &str) -> Option<fpas_vm::DebugDataLocationIdentity> {
    let index = data_id.strip_prefix("g:")?.parse::<u64>().ok()?;
    Some(fpas_vm::DebugDataLocationIdentity::Global { index })
}
