//! DAP source and function breakpoint request translation.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, DebugRecord, DebugRequest, ResponseBody};

impl DapServer {
    pub(super) fn set_source_breakpoints(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        let requested_source = arguments
            .pointer("/source/path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if requested_source.is_empty() {
            return vec![self.failure(
                request_seq,
                "setBreakpoints",
                "setBreakpoints requires source.path.",
            )];
        }
        let source = match self.resolve_source_path(&requested_source) {
            Ok(source) => source,
            Err(message) => {
                return vec![self.failure(request_seq, "setBreakpoints", message)];
            }
        };
        for id in self.source_breakpoints.remove(&source).unwrap_or_default() {
            let core_id = self.next_core_id();
            let _ = self.core.execute(DebugRequest::new(
                core_id,
                DebugOp::BreakpointClear { breakpoint_id: id },
            ));
        }
        let mut ids = Vec::new();
        let mut bodies = Vec::new();
        for requested in arguments
            .get("breakpoints")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            let Some(line) = requested
                .get("line")
                .and_then(Value::as_u64)
                .and_then(|line| u32::try_from(line).ok())
                .filter(|line| *line > 0)
            else {
                bodies.push(json!({
                    "verified": false,
                    "message": "setBreakpoints requires a positive line.",
                    "source": {"path": source},
                    "line": requested.get("line"),
                    "column": requested.get("column")
                }));
                continue;
            };
            let column = requested
                .get("column")
                .and_then(Value::as_u64)
                .and_then(|column| u32::try_from(column).ok());
            let assign = match args::parse_assign(requested.get("assign")) {
                Ok(assign) => assign,
                Err(message) => {
                    bodies.push(json!({
                        "verified": false,
                        "message": message,
                        "source": {"path": source},
                        "line": line,
                        "column": requested.get("column")
                    }));
                    continue;
                }
            };
            let core_id = self.next_core_id();
            let records = self.core.execute(DebugRequest::new(
                core_id,
                DebugOp::BreakpointSet {
                    source: source.clone(),
                    line,
                    column,
                    assign,
                    condition: args::optional_string(&requested, "condition"),
                    hit_condition: args::optional_string(&requested, "hitCondition"),
                    log_message: args::optional_string(&requested, "logMessage"),
                },
            ));
            match records.into_iter().find_map(|record| match record {
                DebugRecord::Response { outcome, .. } => Some(outcome),
                DebugRecord::Event(_) => None,
            }) {
                Some(Ok(ResponseBody::Breakpoint(breakpoint))) => {
                    ids.push(breakpoint.id);
                    bodies.push(json!({
                        "id": breakpoint.id,
                        "verified": breakpoint.is_verified(),
                        "message": (!breakpoint.is_verified()).then_some(
                            "No executable sequence point exists on the requested line."
                        ),
                        "source": {"path": source},
                        "line": breakpoint.location.as_ref().map_or(line, |location| location.line),
                        "column": breakpoint.location.as_ref().map_or(column, |location| Some(location.column))
                    }));
                }
                Some(Ok(ResponseBody::UnverifiedBreakpoint { message, .. })) => {
                    bodies.push(json!({
                        "verified": false,
                        "message": message,
                        "source": {"path": source},
                        "line": line,
                        "column": requested.get("column")
                    }));
                }
                Some(Err(error)) => {
                    bodies.push(json!({
                        "verified": false,
                        "message": error.message,
                        "source": {"path": source},
                        "line": line,
                        "column": requested.get("column")
                    }));
                }
                _ => {
                    bodies.push(json!({
                        "verified": false,
                        "message": "Invalid breakpoint request.",
                        "source": {"path": source},
                        "line": line,
                        "column": requested.get("column")
                    }));
                }
            }
        }
        self.source_breakpoints.insert(source, ids);
        vec![self.success(request_seq, "setBreakpoints", json!({"breakpoints":bodies}))]
    }

    pub(super) fn set_function_breakpoints(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        let requested = arguments
            .get("breakpoints")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if requested.iter().any(|breakpoint| {
            breakpoint.get("logMessage").is_some()
                || breakpoint.get("action").is_some()
                || breakpoint.get("assign").is_some()
        }) {
            return vec![self.failure(
                request_seq,
                "setFunctionBreakpoints",
                "Function breakpoints support only name, condition, and hitCondition; logMessage, assign, and custom actions are unsupported.",
            )];
        }
        match args::parse_function_breakpoints(arguments) {
            Ok(breakpoints) => self.core_request(
                request_seq,
                "setFunctionBreakpoints",
                DebugOp::FunctionBreakpointsReplace { breakpoints },
            ),
            Err(message) => vec![self.failure(request_seq, "setFunctionBreakpoints", &message)],
        }
    }

    pub(super) fn resolve_source_path(&self, requested: &str) -> Result<String, &'static str> {
        self.source_paths.resolve(requested).map_err(
            |_| "Source path is ambiguous; use the exact workspace path or portable debugger path.",
        )
    }
}

pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    let ResponseBody::FunctionBreakpoints { breakpoints } = body else {
        return None;
    };
    (command == "setFunctionBreakpoints").then(|| {
        json!({
            "breakpoints": breakpoints.iter().map(|breakpoint| {
                let location = breakpoint.locations.first();
                json!({
                    "id": breakpoint.id,
                    "verified": breakpoint.is_verified(),
                    "message": function_breakpoint_message(breakpoint),
                    "source": location.map(|location| json!({"path": location.source})),
                    "line": location.map(|location| location.line),
                    "column": location.map(|location| location.column)
                })
            }).collect::<Vec<_>>()
        })
    })
}

fn function_breakpoint_message(breakpoint: &fpas_vm::BoundFunctionBreakpoint) -> Option<String> {
    if breakpoint.functions.is_empty() {
        Some("No executable function metadata matches the requested selector.".to_string())
    } else if breakpoint.instructions.is_empty() {
        Some("Matching functions have no executable entry sequence point.".to_string())
    } else if breakpoint.functions.len() > 1 {
        Some(format!(
            "Bound to {} exact functions in executable order.",
            breakpoint.functions.len()
        ))
    } else {
        None
    }
}
