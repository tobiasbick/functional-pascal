//! DAP runtime-failure filter advertisement and request translation.

use fpas_diagnostics::codes::RUNTIME_ALLOCATED_CODES;
use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::DebugOp;

impl DapServer {
    pub(super) fn set_exception_breakpoints(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        match args::parse_filters(arguments) {
            Ok(filters) => self.core_request(
                request_seq,
                "setExceptionBreakpoints",
                DebugOp::RuntimeFailuresReplace { filters },
            ),
            Err(message) => {
                vec![self.failure(request_seq, "setExceptionBreakpoints", &message)]
            }
        }
    }
}

pub(super) fn advertised_filters() -> Vec<Value> {
    let mut filters = vec![json!({
        "filter": "all",
        "label": "All FPAS runtime failures",
        "description": "Stop on every structured FPAS runtime diagnostic.",
        "default": true
    })];
    filters.extend(RUNTIME_ALLOCATED_CODES.iter().map(|code| {
        json!({
            "filter": code.to_string(),
            "label": format!("{code} runtime failure"),
            "description": format!("Stop only when the runtime diagnostic code is {code}."),
            "default": false
        })
    }));
    filters
}

pub(super) fn response_body(command: &str) -> Option<Value> {
    (command == "setExceptionBreakpoints").then(|| json!({}))
}
