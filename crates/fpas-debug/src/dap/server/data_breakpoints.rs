//! DAP data-breakpoint translation onto durable location identities.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, ResponseBody};

impl DapServer {
    pub(super) fn data_breakpoint_info(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        match (
            args::required_u64(arguments, "variablesReference"),
            args::required_string(arguments, "name"),
        ) {
            (Ok(variables_reference), Ok(name)) => self.core_request(
                request_seq,
                "dataBreakpointInfo",
                DebugOp::LocationDescribe {
                    variables_reference,
                    name,
                },
            ),
            (Err(message), _) | (_, Err(message)) => {
                vec![self.failure(request_seq, "dataBreakpointInfo", &message)]
            }
        }
    }

    pub(super) fn set_data_breakpoints(
        &mut self,
        request_seq: u64,
        arguments: &Value,
    ) -> Vec<Value> {
        match args::parse_data_breakpoints(arguments) {
            Ok(breakpoints) => self.core_request(
                request_seq,
                "setDataBreakpoints",
                DebugOp::DataBreakpointsReplace { breakpoints },
            ),
            Err(message) => vec![self.failure(request_seq, "setDataBreakpoints", &message)],
        }
    }
}

pub(super) fn response_body(command: &str, body: &ResponseBody) -> Option<Value> {
    match (command, body) {
        ("dataBreakpointInfo", ResponseBody::Location(location)) => {
            Some(info_body(location.identity))
        }
        ("setDataBreakpoints", ResponseBody::DataBreakpoints { breakpoints }) => Some(json!({
            "breakpoints": breakpoints.iter().map(|breakpoint| json!({
                "id": breakpoint.id,
                "verified": breakpoint.is_verified(),
                "message": breakpoint.message
            })).collect::<Vec<_>>()
        })),
        _ => None,
    }
}

fn info_body(identity: Option<fpas_vm::DebugDataLocationIdentity>) -> Value {
    match identity {
        Some(fpas_vm::DebugDataLocationIdentity::Global { index }) => json!({
            "dataId": format!("g:{index}"),
            "description": format!("global slot {index}"),
            "accessTypes": ["write"],
            "canPersist": false
        }),
        _ => json!({
            "dataId": Value::Null,
            "description": "Only executable globals are watchable; frame registers and capture cells are not.",
            "accessTypes": ["write"],
            "canPersist": false
        }),
    }
}
