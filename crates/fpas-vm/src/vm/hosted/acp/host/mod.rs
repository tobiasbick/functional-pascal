//! Explicit host adapters: admission precedes effects; response publication stays with the app.
mod files;
pub(in super::super) mod resources;
pub(in super::super) mod wait;

use super::arguments as args;
use crate::vm::{VmError, worker::Worker};
use fpas_acp::{RawJson, binding::Type, connection::Fault, protocol::v1 as sdk, schema::Version};
use fpas_bytecode::{AcpIntrinsic as Op, SourceLocation, Value};
use fpas_std::AggregateFactory;
use std::time::Duration;

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
    fn acp_host_request_for(
        &self,
        version: Version,
        key: &str,
        value: &Value,
    ) -> Result<RawJson, Fault> {
        let mut converter = self.acp_converter(version)?;
        let value = converter.unpack(&Type::Named(key.into()), value, None)?;
        converter.catalog.encode(key, &value).map_err(Into::into)
    }
    fn acp_host_request(&self, key: &str, value: &Value) -> Result<RawJson, Fault> {
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
        let result = (|| {
            if matches!(
                op,
                Op::HostRunTerminalAuthentication | Op::HostV2RunTerminalAuthentication
            ) {
                let [launch, method, timeout, cancellation] = arguments else {
                    return Err(Fault::InvalidArgument("Arguments"));
                };
                let launch = args::launch(launch)?;
                let method: crate::vm::hosted::proc::AcpAuthentication =
                    if op == Op::HostRunTerminalAuthentication {
                        fpas_acp::codec::decode::<sdk::AuthMethodTerminal>(
                            self.acp_host_request("AuthMethodTerminal", method)?,
                        )?
                        .value
                        .into()
                    } else {
                        fpas_acp::codec::decode::<fpas_acp::protocol::v2::AuthMethodTerminal>(
                            self.acp_host_request_for(Version::V2, "AuthMethodTerminal", method)?,
                        )?
                        .value
                        .into()
                    };
                let timeout = args::timeout(timeout)?.map(Duration::from_millis);
                let token = args::optional(cancellation)?
                    .map(args::handle)
                    .transpose()?;
                if let Some(token) = token {
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .map_err(|_| Fault::InvalidArgument("CancellationToken"))?;
                }
                let terminal = self
                    .hosted
                    .process_terminal
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone();
                return crate::vm::hosted::proc::acp_authenticate(
                    &launch,
                    &method,
                    timeout,
                    terminal.as_ref(),
                    &self.hosted.processes,
                    || {
                        self.check_task_cancellation().is_err()
                            || token.is_some_and(|token| {
                                self.hosted
                                    .cancellations
                                    .is_cancelled(token)
                                    .unwrap_or(true)
                            })
                    },
                )
                .map(Value::Boolean);
            }
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
                        "Std.Acp.Types.RawJson",
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
            let (method, key, output) = operation(op).ok_or(Fault::InvalidArgument("Operation"))?;
            let request = self.acp_host_request(key, payload)?;
            let terminals = self.hosted.acp.terminals(connection.id())?;
            let response = if method.starts_with("fs/") {
                files::validate(method, &request)?;
                let _claim = connection.claim_host(responder, method, &request)?;
                files::execute(method, &request, connection.limits().message_bytes)?
            } else if op == Op::HostCreateTerminal {
                let value =
                    fpas_acp::codec::decode::<sdk::CreateTerminalRequest>(request.clone())?.value;
                let launch = crate::vm::hosted::proc::AcpLaunch {
                    command: value.command,
                    arguments: value.args,
                    directory: value.cwd.map(|path| path.to_string_lossy().into_owned()),
                    environment: value
                        .env
                        .into_iter()
                        .map(|value| (value.name, Some(value.value)))
                        .collect(),
                };
                terminals.validate(&launch)?;
                let (_claim, mut reserved) =
                    connection.claim_host_with(responder, method, &request, || {
                        terminals.reserve(&value.session_id.0, value.output_byte_limit)
                    })?;
                terminals.start(&mut reserved, &launch)?;
                raw(&sdk::CreateTerminalResponse::new(reserved.id.clone()))?
            } else {
                let id = member(&request, "terminalId")?;
                let session = member(&request, "sessionId")?;
                terminals.validate_id(&id, &session)?;
                let _claim = connection.claim_host(responder, method, &request)?;
                match op {
                    Op::HostTerminalOutput => raw(&terminals.output(&id, &session)?)?,
                    Op::HostKillTerminal => {
                        terminals.kill(&id, &session)?;
                        RawJson::parse("{}")?
                    }
                    Op::HostReleaseTerminal => {
                        terminals.release(&id, Some(&session))?;
                        RawJson::parse("{}")?
                    }
                    _ => return Err(Fault::InvalidArgument("Operation")),
                }
            };
            if response.text().len().saturating_add(128) > connection.limits().message_bytes {
                return Err(Fault::LimitExceeded(
                    "MessageBytes",
                    connection.limits().message_bytes,
                ));
            }
            self.acp_host_response(output, &response, location)
        })();
        self.acp_result(&op.source_name(), result, location)
    }
}
