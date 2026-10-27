//! Construct FPAS results only on the VM execution lane, after native dequeue.

use super::values::Converter;
use crate::vm::{VmError, worker::Worker};
use fpas_acp::binding::CatalogCodec as _;
use fpas_acp::{
    binding::{self, Type},
    connection::{Closed, Event, Fault, Payload},
    schema::Version,
    transport::ExitStatus,
};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::AggregateFactory;

impl Worker {
    pub(super) fn acp_converter(&self, version: Version) -> Result<Converter<'_>, Fault> {
        Ok(Converter {
            catalog: binding::catalog(version)?,
            image: self.executable.executable(),
            layouts: &self.layouts,
            depth: 0,
        })
    }

    pub(in crate::vm) fn acp_result(
        &self,
        operation: &str,
        result: Result<Value, Fault>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        match result {
            Ok(value) => Ok(Value::result_ok(value)),
            Err(fault) => Ok(Value::result_error(
                self.acp_error(operation, fault, location)?,
            )),
        }
    }

    fn acp_error(
        &self,
        operation: &str,
        fault: Fault,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let (variant, fields) = match fault {
            Fault::InvalidArgument(name) => (
                "InvalidArgument",
                vec![
                    Value::Str(name.into()),
                    Value::Str("Invalid ACP argument".into()),
                ],
            ),
            Fault::LimitExceeded(name, maximum) => (
                "LimitExceeded",
                vec![Value::Str(name.into()), Value::Integer(maximum as i64)],
            ),
            Fault::Busy(operation) => ("Busy", vec![Value::Str(operation.into())]),
            Fault::UnsupportedCapability(name) => {
                ("UnsupportedCapability", vec![Value::Str(name.into())])
            }
            Fault::UnsupportedVersion(version) => (
                "UnsupportedVersion",
                vec![Value::Integer(i64::from(version))],
            ),
            Fault::Closed => ("Closed", vec![Value::Str(operation.into())]),
            Fault::Cancelled => ("Cancelled", vec![Value::Str(operation.into())]),
            Fault::Timeout => ("Timeout", vec![Value::Str(operation.into())]),
            Fault::InvalidMessage => (
                "InvalidMessage",
                vec![
                    Value::OptionNone,
                    Value::OptionNone,
                    Value::Str("Invalid ACP message".into()),
                ],
            ),
            Fault::Host => (
                "Host",
                vec![fpas_std::native_errors::host(
                    self.layouts.as_ref(),
                    operation,
                    &[],
                    None,
                    "ACP host operation failed".into(),
                    location,
                )?],
            ),
            Fault::Peer(raw) => {
                let mut converter = self.acp_converter(Version::V1).map_err(|_| {
                    crate::vm::diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "ACP catalog unavailable",
                    )
                })?;
                let value = converter
                    .catalog
                    .decode("Error", &raw)
                    .and_then(|value| converter.pack(&Type::Named("Error".into()), value, None));
                match value {
                    Ok(value) => ("Peer", vec![value]),
                    Err(_) => (
                        "InvalidMessage",
                        vec![
                            Value::OptionNone,
                            Value::OptionNone,
                            Value::Str("Invalid ACP peer error".into()),
                        ],
                    ),
                }
            }
        };
        self.layouts
            .enumeration("Std.Acp.Types.AcpError", variant, fields, location)
    }

    pub(super) fn acp_event(
        &self,
        event: Event,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let typed = |version: Version, root: &str, payload| {
            self.acp_converter(version)
                .and_then(|mut converter| {
                    converter
                        .pack(&Type::Named(root.into()), payload, None)
                        .map_err(Fault::from)
                })
                .map_err(|_| {
                    crate::vm::diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "ACP event does not match its retained layout",
                    )
                })
        };
        let (variant, fields) = match event.payload {
            Payload::McpResponse(token, version, payload) => (
                if version == Version::V1 {
                    "McpResponse"
                } else {
                    "V2McpResponse"
                },
                vec![
                    Value::OpaqueHandle(token.id()),
                    typed(version, "MessageMcpResponse", payload)?,
                ],
            ),
            Payload::Initialized(token, version, payload) => {
                let response = typed(version, "InitializeResponse", payload)?;
                let outcome = if version == Version::V2 {
                    self.layouts.enumeration(
                        "Std.Acp.V2.InitializeOutcome",
                        "Initialized",
                        vec![response],
                        location,
                    )?
                } else {
                    let response = self.layouts.enumeration(
                        "Std.Acp.V2.SelectedInitializeResponse",
                        "V1",
                        vec![response],
                        location,
                    )?;
                    let mismatch = self.layouts.record(
                        "Std.Acp.V2.InitializationVersionMismatch",
                        vec![Value::Integer(2), Value::Integer(1), response],
                        location,
                    )?;
                    self.layouts.enumeration(
                        "Std.Acp.V2.InitializeOutcome",
                        "LowerVersion",
                        vec![mismatch],
                        location,
                    )?
                };
                (
                    "V2Initialized",
                    vec![Value::OpaqueHandle(token.id()), outcome],
                )
            }
            Payload::Response(token, version, payload) => (
                if version == Version::V1 {
                    "Response"
                } else {
                    "V2Response"
                },
                vec![
                    Value::OpaqueHandle(token.id()),
                    typed(version, "AgentResponse", payload)?,
                ],
            ),
            Payload::Notification(version, payload) => (
                if version == Version::V1 {
                    "Notification"
                } else {
                    "V2Notification"
                },
                vec![typed(version, "AgentNotification", payload)?],
            ),
            Payload::Request(responder, version, payload) => (
                if version == Version::V1 {
                    "Request"
                } else {
                    "V2Request"
                },
                vec![
                    typed(version, "AgentRequest", payload)?,
                    Value::OpaqueHandle(responder.id()),
                ],
            ),
            Payload::RequestFailed(token, error) => (
                "RequestFailed",
                vec![
                    Value::OpaqueHandle(token.id()),
                    self.acp_error("Request", error, location)?,
                ],
            ),
            Payload::RequestCancelled(responder) => (
                "RequestCancelled",
                vec![Value::OpaqueHandle(responder.id())],
            ),
            Payload::UnknownNotification(method, raw) => (
                "UnknownNotification",
                vec![
                    Value::Str(method.into()),
                    self.layouts.record(
                        "Std.Acp.Types.RawJson",
                        vec![Value::Str(raw.text().into())],
                        location,
                    )?,
                ],
            ),
            Payload::Closed(closed) => ("Closed", vec![self.acp_closed(closed, location)?]),
        };
        let payload =
            self.layouts
                .enumeration("Std.Acp.EventPayload", variant, fields, location)?;
        let raw = match event.raw {
            Some(raw) => Value::option_some(self.layouts.record(
                "Std.Acp.Types.RawJson",
                vec![Value::Str(raw.text().into())],
                location,
            )?),
            None => Value::OptionNone,
        };
        self.layouts
            .record("Std.Acp.Event", vec![payload, raw], location)
    }

    fn acp_closed(&self, closed: Closed, location: SourceLocation) -> Result<Value, VmError> {
        let error = match closed.fault {
            Some(error) => Value::option_some(self.acp_error("Connection", error, location)?),
            None => Value::OptionNone,
        };
        let status = match closed.transport.exit {
            Some(status) => {
                let (variant, fields) = match status {
                    ExitStatus::Exited(code) => ("Exited", vec![Value::Integer(i64::from(code))]),
                    ExitStatus::Signaled(signal) => {
                        ("Signaled", vec![Value::Integer(i64::from(signal))])
                    }
                    ExitStatus::Unknown => ("Unknown", vec![]),
                };
                Value::option_some(self.layouts.enumeration(
                    "Std.Acp.ProcessExitStatus",
                    variant,
                    fields,
                    location,
                )?)
            }
            None => Value::OptionNone,
        };
        self.layouts.record(
            "Std.Acp.ClosedStatus",
            vec![
                error,
                status,
                Value::Str(closed.transport.stderr.into()),
                Value::Boolean(closed.transport.stderr_truncated),
            ],
            location,
        )
    }
}
