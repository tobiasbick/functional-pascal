//! DAP source and function breakpoint request translation.

use serde_json::{Value, json};

use super::{DapServer, core_request};

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
        let source = self.resolve_source_path(&requested_source);
        for id in self.source_breakpoints.remove(&source).unwrap_or_default() {
            let core_id = self.next_core_id();
            let _ = self.core.handle_line(&core_request(
                core_id,
                "breakpoint.clear",
                json!({"breakpoint_id":id}),
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
            let core_id = self.next_core_id();
            let records = self.core.handle_line(&core_request(
                core_id,
                "breakpoint.set",
                json!({
                    "source":source,
                    "line":requested.get("line"),
                    "column":requested.get("column"),
                    "condition":requested.get("condition"),
                    "hit_condition":requested.get("hitCondition"),
                    "log_message":requested.get("logMessage"),
                    "assign":requested.get("assign")
                }),
            ));
            if records
                .first()
                .is_some_and(|record| record["success"] == false)
            {
                let message = records[0]["error"]["message"]
                    .as_str()
                    .unwrap_or("Invalid breakpoint request.");
                return vec![self.failure(request_seq, "setBreakpoints", message)];
            }
            if let Some(body) = records.first().and_then(|record| record.get("body")) {
                if let Some(id) = body.get("breakpoint_id").and_then(Value::as_u64) {
                    ids.push(id);
                }
                bodies.push(json!({
                    "id": body.get("breakpoint_id"),
                    "verified": body.get("verified"),
                    "message": body.get("message"),
                    "source": {"path": source},
                    "line": body.pointer("/location/line").or_else(|| requested.get("line")),
                    "column": body.pointer("/location/column").or_else(|| requested.get("column"))
                }));
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
        let breakpoints = requested
            .into_iter()
            .map(|breakpoint| {
                json!({
                    "name": breakpoint.get("name"),
                    "condition": breakpoint.get("condition"),
                    "hit_condition": breakpoint.get("hitCondition")
                })
            })
            .collect::<Vec<_>>();
        self.core_request(
            request_seq,
            "setFunctionBreakpoints",
            "function_breakpoints.replace",
            json!({"breakpoints": breakpoints}),
        )
    }

    pub(super) fn resolve_source_path(&self, requested: &str) -> String {
        let normalized = requested.replace('\\', "/");
        let mut matches = self
            .source_paths
            .iter()
            .filter(|source| normalized == **source || normalized.ends_with(&format!("/{source}")));
        let first = matches.next();
        if first.is_some() && matches.next().is_none() {
            return first.cloned().unwrap_or_else(|| requested.to_string());
        }
        requested.to_string()
    }
}

pub(super) fn response_body(command: &str, body: &Value) -> Option<Value> {
    (command == "setFunctionBreakpoints").then(|| {
        let breakpoints = body
            .get("breakpoints")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|breakpoint| {
                let location = breakpoint
                    .get("locations")
                    .and_then(Value::as_array)
                    .and_then(|locations| locations.first());
                json!({
                    "id": breakpoint.get("breakpoint_id"),
                    "verified": breakpoint.get("verified"),
                    "message": breakpoint.get("message"),
                    "source": location.and_then(|location| location.get("source")).map(|path| json!({"path": path})),
                    "line": location.and_then(|location| location.get("line")),
                    "column": location.and_then(|location| location.get("column"))
                })
            })
            .collect::<Vec<_>>();
        json!({"breakpoints": breakpoints})
    })
}
