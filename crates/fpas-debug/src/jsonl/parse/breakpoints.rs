//! JSONL parsing for source, function, and data breakpoints.

use serde_json::{Map, Value};

use super::args::{missing, optional_string, required_string};
use super::identity::parse_identity;
use crate::engine::{AssignOp, DataBreakpointOp, DebugOp, EngineFailure, FunctionBreakpointOp};

pub(super) fn parse_breakpoint_set(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    let source = required_string(command, arguments, "source")?;
    let Some(line) = arguments
        .get("line")
        .and_then(Value::as_u64)
        .and_then(|line| u32::try_from(line).ok())
        .filter(|line| *line > 0)
    else {
        return Err(missing(command, "line"));
    };
    let column = arguments
        .get("column")
        .and_then(Value::as_u64)
        .and_then(|column| u32::try_from(column).ok());
    Ok(DebugOp::BreakpointSet {
        source,
        line,
        column,
        assign: parse_assign(arguments.get("assign"))?,
        condition: optional_string(arguments, "condition"),
        hit_condition: optional_string(arguments, "hit_condition"),
        log_message: optional_string(arguments, "log_message"),
    })
}

pub(super) fn parse_assign(value: Option<&Value>) -> Result<Option<AssignOp>, EngineFailure> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(object) = value.as_object() else {
        return Err(EngineFailure::new(
            "invalid_request",
            "Command `assign` must be an object with `identity` and `expression`.",
            "Send `assign.identity` from `location.describe` and one replacement `expression`.",
        ));
    };
    if object
        .keys()
        .any(|field| !matches!(field.as_str(), "identity" | "expression"))
    {
        return Err(EngineFailure::new(
            "invalid_request",
            "Command `assign` contains an unsupported field.",
            "Send `assign.identity` from `location.describe` and one replacement `expression`.",
        ));
    }
    let Some(identity) = object.get("identity").and_then(parse_identity) else {
        return Err(EngineFailure::new(
            "invalid_request",
            "Command `assign` requires a location identity from `location.describe`.",
            "Send `assign.identity` from `location.describe` and one replacement `expression`.",
        ));
    };
    let Some(expression) = object.get("expression").and_then(Value::as_str) else {
        return Err(EngineFailure::new(
            "invalid_request",
            "Command `assign` requires string field `expression`.",
            "Send `assign.identity` from `location.describe` and one replacement `expression`.",
        ));
    };
    Ok(Some(AssignOp {
        identity,
        expression: expression.to_string(),
    }))
}

pub(super) fn parse_function_breakpoints(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<Vec<FunctionBreakpointOp>, EngineFailure> {
    let Some(requested) = arguments.get("breakpoints").and_then(Value::as_array) else {
        return Err(missing(command, "breakpoints"));
    };
    requested
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let Some(item) = item.as_object() else {
                return Err(invalid_function(index, "expected an object"));
            };
            if item
                .keys()
                .any(|field| !matches!(field.as_str(), "name" | "condition" | "hit_condition"))
            {
                return Err(invalid_function(index, "contains an unsupported field"));
            }
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                return Err(invalid_function(index, "requires string field `name`"));
            };
            Ok(FunctionBreakpointOp {
                name: name.to_string(),
                condition: optional_string(item, "condition"),
                hit_condition: optional_string(item, "hit_condition"),
            })
        })
        .collect()
}

pub(super) fn parse_data_breakpoints(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<Vec<DataBreakpointOp>, EngineFailure> {
    let Some(requested) = arguments.get("breakpoints").and_then(Value::as_array) else {
        return Err(missing(command, "breakpoints"));
    };
    requested
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let Some(item) = item.as_object() else {
                return Err(invalid_data(index, "expected an object"));
            };
            if item
                .keys()
                .any(|field| !matches!(field.as_str(), "identity" | "access" | "assign"))
            {
                return Err(invalid_data(index, "contains an unsupported field"));
            }
            let Some(identity) = item.get("identity").and_then(parse_identity) else {
                return Err(invalid_data(
                    index,
                    "requires a location identity from `location.describe`",
                ));
            };
            let access = match item.get("access").and_then(Value::as_str) {
                None => fpas_vm::DataBreakpointAccess::Write,
                Some(value) => fpas_vm::DataBreakpointAccess::parse(value)
                    .ok_or_else(|| invalid_data(index, "access must be write, change, or read"))?,
            };
            let assign = parse_assign(item.get("assign")).map_err(|error| {
                EngineFailure::new(
                    "invalid_request",
                    format!("Data breakpoint at index {index}: {}", error.message),
                    "Send `assign.identity` from `location.describe` and one replacement `expression`.",
                )
            })?;
            Ok(DataBreakpointOp {
                identity,
                access,
                assign,
            })
        })
        .collect()
}

fn invalid_function(index: usize, detail: &str) -> EngineFailure {
    EngineFailure::new(
        "invalid_request",
        format!("Function breakpoint at index {index} {detail}."),
        "Send a bounded array of names with optional condition and hit_condition strings.",
    )
}

fn invalid_data(index: usize, detail: &str) -> EngineFailure {
    EngineFailure::new(
        "invalid_request",
        format!("Data breakpoint at index {index} {detail}."),
        "Send identities from `location.describe` with access write or change.",
    )
}
