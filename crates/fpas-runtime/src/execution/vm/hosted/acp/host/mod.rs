//! Explicit host adapters: admission precedes effects; response publication stays with the app.
pub(super) mod files;
pub(in super::super) mod resources;

use super::arguments as args;
use crate::execution::vm::{VmError, worker::Worker};
use crate::stdlib::AggregateFactory;
use crate::{AcpIntrinsic as Op, SourceLocation, Value};
use fpas_acp::binding::CatalogCodec as _;
use fpas_acp::{RawJson, binding::Type, connection::Fault, schema::Version};

pub(super) fn operation(op: Op) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match op {
        Op::HostReadTextFile => (
            "fs/read_text_file",
            "ReadTextFileRequest",
            "ReadTextFileResponse",
        ),
        Op::HostWriteTextFile => (
            "fs/write_text_file",
            "WriteTextFileRequest",
            "WriteTextFileResponse",
        ),
        Op::HostCreateTerminal => (
            "terminal/create",
            "CreateTerminalRequest",
            "CreateTerminalResponse",
        ),
        Op::HostTerminalOutput => (
            "terminal/output",
            "TerminalOutputRequest",
            "TerminalOutputResponse",
        ),
        Op::HostWaitForTerminalExit => (
            "terminal/wait_for_exit",
            "WaitForTerminalExitRequest",
            "WaitForTerminalExitResponse",
        ),
        Op::HostKillTerminal => (
            "terminal/kill",
            "KillTerminalRequest",
            "KillTerminalResponse",
        ),
        Op::HostReleaseTerminal => (
            "terminal/release",
            "ReleaseTerminalRequest",
            "ReleaseTerminalResponse",
        ),
        _ => return None,
    })
}
pub(super) fn raw(value: &impl serde::Serialize) -> Result<RawJson, Fault> {
    RawJson::parse(&serde_json::to_string(value).map_err(|_| Fault::InvalidMessage)?)
        .map_err(Into::into)
}
pub(super) fn member(raw: &RawJson, name: &str) -> Result<String, Fault> {
    let value = raw
        .member(name)?
        .ok_or(Fault::InvalidArgument("HostRequest"))?;
    serde_json::from_str(value.text()).map_err(|_| Fault::InvalidArgument("HostRequest"))
}
impl Worker {
    pub(in super::super) fn acp_host_request_for(
        &self,
        version: Version,
        key: &str,
        value: &Value,
    ) -> Result<RawJson, Fault> {
        let mut converter = self.acp_converter(version)?;
        let value = converter.unpack(&Type::Named(key.into()), value, None)?;
        converter.catalog.encode(key, &value).map_err(Into::into)
    }
    pub(in super::super) fn acp_host_request(
        &self,
        key: &str,
        value: &Value,
    ) -> Result<RawJson, Fault> {
        self.acp_host_request_for(Version::V1, key, value)
    }
    pub(in super::super) fn acp_host_response(
        &self,
        key: &str,
        raw: &RawJson,
        _location: SourceLocation,
    ) -> Result<Value, Fault> {
        let mut converter = self.acp_converter(Version::V1)?;
        let value = converter.catalog.decode(key, raw)?;
        converter
            .pack(&Type::Named(key.into()), value, None)
            .map_err(Into::into)
    }
    pub(super) fn execute_acp_host(
        &self,
        op: Op,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        if super::native::supported(op) {
            return self.execute_acp_native(op, arguments, location);
        }
        let result = (|| {
            if op == Op::HostElicitationDefaults {
                let [responder] = arguments else {
                    return Err(Fault::InvalidArgument("Arguments"));
                };
                let (connection, responder) =
                    self.hosted.acp.responder(args::handle(responder)?)?;
                let request = connection.incoming(responder, "elicitation/create")?;
                let value = fpas_acp::interaction::defaults(&request)?;
                return self
                    .layouts
                    .record(
                        "Std.Ai.Acp.Types.RawJson",
                        vec![Value::Str(value.text().into())],
                        location,
                    )
                    .map_err(|_| Fault::Host);
            }
            let [owner, payload] = arguments else {
                return Err(Fault::InvalidArgument("Arguments"));
            };
            let (connection, responder) = self.hosted.acp.responder(args::handle(owner)?)?;
            if op == Op::HostValidateElicitation {
                let response = self.acp_host_request("CreateElicitationResponse", payload)?;
                return connection
                    .validate_interaction(responder, &response)
                    .map(Value::Boolean);
            }
            Err(Fault::InvalidArgument("Operation"))
        })();
        self.acp_result(&op.source_name(), result, location)
    }
}
