//! Shared JSONL argument extractors.

use serde_json::{Map, Value};

use crate::engine::EngineFailure;

pub(super) fn missing(command: &str, argument: &str) -> EngineFailure {
    EngineFailure::new(
        "invalid_request",
        format!("Command `{command}` requires argument `{argument}`."),
        "Add the required field to the request `arguments` object.",
    )
}

pub(super) fn required_string(
    command: &str,
    arguments: &Map<String, Value>,
    name: &str,
) -> Result<String, EngineFailure> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| missing(command, name))
}

pub(super) fn require_string_typed(
    command: &str,
    arguments: &Map<String, Value>,
    name: &str,
) -> Result<String, EngineFailure> {
    match arguments.get(name) {
        None => Err(missing(command, name)),
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(EngineFailure::new(
            "invalid_request",
            format!("Command `{command}` argument `{name}` must be a string."),
            "Pass one FPAS expression string.",
        )),
    }
}

pub(super) fn optional_string(arguments: &Map<String, Value>, name: &str) -> Option<String> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(super) fn optional_expression_string(
    arguments: &Map<String, Value>,
    command_name: &str,
) -> Result<Option<String>, EngineFailure> {
    match arguments.get("expression") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(source)) => Ok(Some(source.clone())),
        Some(_) => Err(EngineFailure::new(
            "invalid_request",
            format!(
                "Command `{command_name}` argument `expression` must be a string when present."
            ),
            if command_name == "task.result.replace" {
                "Omit `expression` for procedure tasks, or pass one FPAS expression string for function tasks."
            } else {
                "Omit `expression` for procedures, or pass one FPAS expression string for functions."
            },
        )),
    }
}

pub(super) fn optional_u64(
    command: &str,
    arguments: &Map<String, Value>,
    name: &str,
) -> Result<Option<u64>, EngineFailure> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            let help = if name == "task_id" {
                "Pass a task ID returned by `tasks` as `task_id`.".to_string()
            } else {
                format!(
                    "Pass a non-negative ID returned by the matching inspection request as `{name}`."
                )
            };
            EngineFailure::new(
                "invalid_request",
                format!("Command `{command}` argument `{name}` must be a non-negative integer."),
                help,
            )
        }),
    }
}

pub(super) fn required_u64(
    command: &str,
    arguments: &Map<String, Value>,
    name: &str,
) -> Result<u64, EngineFailure> {
    optional_u64(command, arguments, name)?.ok_or_else(|| missing(command, name))
}

pub(super) fn index_argument(arguments: &Map<String, Value>, name: &str, default: usize) -> usize {
    arguments
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(default)
}

pub(super) fn parse_string_array(
    command: &str,
    arguments: &Map<String, Value>,
    name: &str,
) -> Result<Vec<String>, EngineFailure> {
    let Some(requested) = arguments.get(name).and_then(Value::as_array) else {
        return Err(missing(command, name));
    };
    requested
        .iter()
        .enumerate()
        .map(|(index, filter)| {
            filter.as_str().map(str::to_string).ok_or_else(|| {
                EngineFailure::new(
                    "invalid_request",
                    format!("Runtime failure filter at index {index} must be a string."),
                    "Use `all` or exact advertised codes such as `F4001`.",
                )
            })
        })
        .collect()
}
