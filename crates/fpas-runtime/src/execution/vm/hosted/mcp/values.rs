//! Bounded JSON envelopes and nominal error/context records.

use super::Failure;
use crate::execution::vm::{VmError, worker::Worker};
use crate::stdlib::AggregateFactory;
use crate::{SourceLocation, Value};
use fpas_mcp::{Fault, HttpError, model::CallToolResult};

impl Worker {
    fn mcp_serialized_raw(
        &self,
        text: Result<String, serde_json::Error>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let text = text.map_err(|_| {
            crate::execution::vm::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "MCP value could not be serialized",
            )
        })?;
        self.mcp_raw(&text, location)
    }
    pub(super) fn mcp_raw(&self, text: &str, location: SourceLocation) -> Result<Value, VmError> {
        self.layouts.record(
            "Std.Mcp.Types.RawJson",
            vec![Value::Str(text.into())],
            location,
        )
    }
    pub(super) fn mcp_tool_result(
        &self,
        result: &CallToolResult,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let raw = self.mcp_serialized_raw(serde_json::to_string(result), location)?;
        self.layouts
            .record("Std.Mcp.Types.ToolResult", vec![raw], location)
    }
    pub(super) fn mcp_result(
        &self,
        result: Result<Value, Failure>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let error = match result {
            Ok(v) => return Ok(Value::result_ok(v)),
            Err(e) => e,
        };
        let (name, fields) = match error {
            Failure::Transport(HttpError::Server(error)) | Failure::Core(error) => match error {
                Fault::InvalidArgument(s) => ("InvalidArgument", vec![Value::Str(s.into())]),
                Fault::InvalidSchema(s) => ("InvalidSchema", vec![Value::Str(s.into())]),
                Fault::DuplicateTool => ("DuplicateTool", vec![]),
                Fault::UnknownTool => ("UnknownTool", vec![]),
                Fault::InvalidArguments => ("InvalidArguments", vec![]),
                Fault::InvalidResult => ("InvalidResult", vec![]),
                Fault::LimitExceeded(s, n) => (
                    "LimitExceeded",
                    vec![Value::Str(s.into()), Value::Integer(n as i64)],
                ),
                Fault::Busy => ("Busy", vec![]),
                Fault::Closed => ("Closed", vec![]),
            },
            Failure::Cancelled => ("Cancelled", vec![]),
            Failure::Transport(error) => {
                ("Transport", vec![Value::Str(format!("{error:?}").into())])
            }
        };
        Ok(Value::result_error(self.layouts.enumeration(
            "Std.Mcp.Types.McpError",
            name,
            fields,
            location,
        )?))
    }
    pub(super) fn mcp_context(
        &self,
        server: u64,
        id: u64,
        call: &fpas_mcp::Call,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let option_text = |s: Option<String>| {
            s.map_or(Value::OptionNone, |s| {
                Value::option_some(Value::Str(s.into()))
            })
        };
        let request = self.mcp_serialized_raw(serde_json::to_string(&call.request_id), location)?;
        let meta = self.mcp_serialized_raw(serde_json::to_string(&call.metadata), location)?;
        self.layouts.record(
            "Std.Mcp.Types.RequestContext",
            vec![
                Value::OpaqueHandle(server),
                Value::Integer(call.responder.endpoint().unwrap_or(0) as i64),
                Value::Integer(id as i64),
                request,
                meta,
                option_text(call.protocol_version.as_ref().map(ToString::to_string)),
                call.connection_id.map_or(Value::OptionNone, |id| {
                    Value::option_some(Value::Integer(id as i64))
                }),
                option_text(call.session_id.clone()),
            ],
            location,
        )
    }
}
