//! Logical breakpoint configuration for the JSONL server.

use serde_json::{Map, Value, json};

use super::{JsonlServer, ServerStatus};
use crate::breakpoints::{BreakpointAssign, BreakpointPolicy};
use crate::evaluation::parse_debug_expression;
use crate::jsonl::encode::{breakpoint_body, invalid_state, missing_argument};
use crate::jsonl::protocol::{event, failure, session_error, success};
use crate::jsonl::server::location::parse_identity;

impl JsonlServer {
    pub(super) fn set_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(source) = arguments.get("source").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "source")];
        };
        let Some(line) = arguments
            .get("line")
            .and_then(Value::as_u64)
            .and_then(|line| u32::try_from(line).ok())
            .filter(|line| *line > 0)
        else {
            return vec![missing_argument(request_id, command, "line")];
        };
        let column = arguments
            .get("column")
            .and_then(Value::as_u64)
            .and_then(|column| u32::try_from(column).ok());
        let assign = match parse_assign_argument(arguments.get("assign")) {
            Ok(assign) => assign,
            Err(message) => {
                return vec![failure(
                    request_id,
                    command,
                    "invalid_request",
                    message,
                    "Send `assign.identity` from `location.describe` and one replacement `expression`.",
                )];
            }
        };
        let policy = match BreakpointPolicy::parse(
            arguments.get("condition").and_then(Value::as_str),
            arguments.get("hit_condition").and_then(Value::as_str),
            arguments.get("log_message").and_then(Value::as_str),
            assign,
        ) {
            Ok(policy) => policy,
            Err(error) => {
                return vec![success(
                    request_id,
                    command,
                    json!({
                        "verified": false,
                        "message": format!("{} Help: {}", error.message, error.hint),
                        "error_code": error.code,
                        "error_offset": error.offset,
                        "error_length": error.length,
                        "requested": {"source": source, "line": line, "column": column}
                    }),
                )];
            }
        };
        let breakpoint = match self.actor.session_mut().map(|session| {
            session.set_breakpoint(fpas_vm::SourceBreakpoint {
                source: source.to_string(),
                line,
                column,
            })
        }) {
            Some(Ok(breakpoint)) => breakpoint,
            Some(Err(error)) => return vec![session_error(request_id, command, error)],
            None => return vec![invalid_state(request_id, command, self.status)],
        };
        self.breakpoint_policies.insert(breakpoint.id, policy);
        let body = breakpoint_body(&breakpoint);
        vec![
            success(request_id, command, body.clone()),
            event("breakpoint", body),
        ]
    }

    pub(super) fn clear_breakpoint(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if !matches!(
            self.status,
            ServerStatus::Initialized | ServerStatus::Stopped
        ) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(id) = arguments.get("breakpoint_id").and_then(Value::as_u64) else {
            return vec![missing_argument(request_id, command, "breakpoint_id")];
        };
        self.breakpoint_policies.remove(&id);
        match self
            .actor
            .session_mut()
            .map(|session| session.clear_breakpoint(id))
        {
            Some(Ok(())) => vec![success(request_id, command, json!({"breakpoint_id": id}))],
            Some(Err(error)) => vec![session_error(request_id, command, error)],
            None => vec![invalid_state(request_id, command, self.status)],
        }
    }
}

pub(super) fn parse_assign_argument(
    value: Option<&Value>,
) -> Result<Option<BreakpointAssign>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(object) = value.as_object() else {
        return Err(
            "Command `assign` must be an object with `identity` and `expression`.".to_string(),
        );
    };
    if object
        .keys()
        .any(|field| !matches!(field.as_str(), "identity" | "expression"))
    {
        return Err("Command `assign` contains an unsupported field.".to_string());
    }
    let Some(identity) = object.get("identity").and_then(parse_identity) else {
        return Err(
            "Command `assign` requires a location identity from `location.describe`.".to_string(),
        );
    };
    let Some(source) = object.get("expression").and_then(Value::as_str) else {
        return Err("Command `assign` requires string field `expression`.".to_string());
    };
    let expression = parse_debug_expression(source, fpas_vm::DebugEvaluationLimits::default())
        .map_err(|error| format!("{} Help: {}", error.message, error.hint))?;
    BreakpointAssign::new(identity, expression).map(Some)
}
