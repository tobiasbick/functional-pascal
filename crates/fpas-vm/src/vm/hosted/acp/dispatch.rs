//! Public connection operations; native queues never contain FPAS values.

use super::arguments as args;
use crate::vm::{VmError, worker::Worker};
use fpas_acp::binding::CatalogCodec as _;
use fpas_acp::{
    RawJson,
    binding::{self, Type, Value as Detached},
    connection::{Fault, Limits, Settings},
    schema::Version,
};
use fpas_bytecode::{AcpIntrinsic as Op, SourceLocation, Value};
use fpas_std::AggregateFactory;

impl Worker {
    pub(super) fn execute_acp_connection(
        &self,
        operation: Op,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        if (789..=798).contains(&(operation as u16))
            || operation == Op::HostV2RunTerminalAuthentication
        {
            return self.execute_acp_host(operation, arguments, location);
        }
        if operation == Op::DefaultOptions {
            return self.acp_default_options(location);
        }
        let result: Result<Value, Fault> = (|| {
            Ok(match (operation, arguments) {
                (Op::SendMcp | Op::V2SendMcp, [endpoint, payload]) => {
                    let endpoint = args::handle(endpoint)?;
                    let connection = self.hosted.acp.mcp_connection(endpoint)?;
                    let version = if operation == Op::SendMcp {
                        Version::V1
                    } else {
                        Version::V2
                    };
                    let mut converter = self.acp_converter(version)?;
                    let value = converter.unpack(
                        &Type::Named("MessageMcpRequest".into()),
                        payload,
                        None,
                    )?;
                    let raw = converter.catalog.encode("MessageMcpRequest", &value)?;
                    Value::OpaqueHandle(connection.send_mcp(endpoint, version, raw)?.id())
                }
                (Op::RegisterMcpEndpoint, [owner, server, session]) => {
                    let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                    Value::OpaqueHandle(connection.register_mcp_endpoint(
                        args::text(server)?,
                        args::optional(session)?.map(args::text).transpose()?,
                    )?)
                }
                (Op::RemoveMcpEndpoint, [owner, endpoint]) => {
                    let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                    Value::Boolean(connection.remove_mcp_endpoint(args::handle(endpoint)?)?)
                }
                (Op::ConnectMessages, [options]) => {
                    let (version, limits) = self.acp_options(options)?;
                    let (connection, endpoint) = self.hosted.acp.connect_messages(
                        version,
                        limits,
                        self.hosted.piped_processes.clone(),
                    )?;
                    return self
                        .layouts
                        .record(
                            "Std.Acp.MessageConnection",
                            vec![
                                Value::OpaqueHandle(connection),
                                Value::OpaqueHandle(endpoint),
                            ],
                            location,
                        )
                        .map_err(|_| Fault::Host);
                }
                (Op::ConnectProcess, [launch, options]) => {
                    let (version, limits) = self.acp_options(options)?;
                    let launch = args::launch(launch)?;
                    Value::OpaqueHandle(self.hosted.acp.connect_process(
                        version,
                        limits,
                        launch,
                        self.hosted.piped_processes.clone(),
                    )?)
                }
                (Op::FeedMessage, [endpoint, message]) => Value::Boolean(
                    self.hosted
                        .acp
                        .endpoint(args::handle(endpoint)?)?
                        .feed(args::raw(message)?)?,
                ),
                (Op::TryTakeMessage, [endpoint]) => {
                    return match self
                        .hosted
                        .acp
                        .endpoint(args::handle(endpoint)?)?
                        .try_take()?
                    {
                        Some(raw) => self
                            .layouts
                            .record(
                                "Std.Acp.Types.RawJson",
                                vec![Value::Str(raw.text().into())],
                                location,
                            )
                            .map(Value::option_some)
                            .map_err(|_| Fault::Host),
                        None => Ok(Value::OptionNone),
                    };
                }
                (Op::CloseInput, [endpoint]) => Value::Boolean(
                    self.hosted
                        .acp
                        .endpoint(args::handle(endpoint)?)?
                        .close_input(),
                ),
                (Op::Send | Op::V2Send | Op::Notify | Op::V2Notify, [owner, payload]) => {
                    let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                    let version = if matches!(operation, Op::V2Send | Op::V2Notify) {
                        Version::V2
                    } else {
                        Version::V1
                    };
                    if connection.version() != version {
                        return Err(Fault::InvalidArgument("ProtocolVersion"));
                    }
                    let root = if matches!(operation, Op::Send | Op::V2Send) {
                        "ClientRequest"
                    } else {
                        "ClientNotification"
                    };
                    let (method, raw) = self.acp_encode(version, root, payload)?;
                    if root == "ClientRequest" {
                        self.prepare_acp_tools(&connection, &method)?;
                        Value::OpaqueHandle(connection.send_for(version, &method, raw)?.id())
                    } else {
                        Value::Boolean(connection.notify_for(version, &method, raw)?)
                    }
                }
                (Op::Respond | Op::V2Respond | Op::RespondError, [owner, payload]) => {
                    let (connection, responder) =
                        self.hosted.acp.responder(args::handle(owner)?)?;
                    if operation == Op::RespondError {
                        let mut converter = self.acp_converter(Version::V1)?;
                        let value =
                            converter.unpack(&Type::Named("Error".into()), payload, None)?;
                        let raw = converter.catalog.encode("Error", &value)?;
                        Value::Boolean(connection.respond_error(responder, raw)?)
                    } else {
                        let version = if operation == Op::V2Respond {
                            Version::V2
                        } else {
                            Version::V1
                        };
                        if connection.responder_version(responder)? != version {
                            return Err(Fault::InvalidArgument("ProtocolVersion"));
                        }
                        let method = connection.responder_method(responder)?;
                        let mut converter = self.acp_converter(version)?;
                        let value = converter
                            .unpack(&Type::Named("ClientResponse".into()), payload, None)
                            .map_err(|_| Fault::InvalidArgument("Response"))?;
                        let raw = converter
                            .catalog
                            .encode_message("ClientResponse", &method, &value)
                            .map_err(|_| Fault::InvalidArgument("Response"))?;
                        Value::Boolean(connection.respond(responder, raw)?)
                    }
                }
                (Op::CancelRequest, [owner, token]) => {
                    let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                    Value::Boolean(
                        connection.cancel(connection.request_token(args::handle(token)?)?)?,
                    )
                }
                (Op::TryReceive, [owner]) => {
                    let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                    return match connection.try_receive()? {
                        Some(event) => self
                            .acp_event(event, location)
                            .map(Value::option_some)
                            .map_err(|_| Fault::Host),
                        None => Ok(Value::OptionNone),
                    };
                }
                (Op::State, [owner]) => {
                    let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                    return self.acp_state(connection.snapshot()?, location);
                }
                _ => return Err(Fault::InvalidArgument("Arguments")),
            })
        })();
        self.acp_result(&operation.source_name(), result, location)
    }

    fn acp_options(&self, options: &Value) -> Result<(Settings, Limits), Fault> {
        let (_, limits) = args::options(options)?;
        let fields = args::record(options, "Std.Acp.Options", 6)?;
        let mut converter = self.acp_converter(Version::V1)?;
        let info = converter.unpack(
            &Type::Option(Box::new(Type::Named("Implementation".into()))),
            &fields[0],
            None,
        )?;
        let caps = converter.unpack(&Type::Named("ClientCapabilities".into()), &fields[1], None)?;
        let raw = converter.catalog.encode("ClientCapabilities", &caps)?;
        let client_capabilities = fpas_acp::codec::decode(raw)?.value;
        let client_info = match info {
            Detached::None => None,
            Detached::Some(info) => Some(
                fpas_acp::codec::decode(converter.catalog.encode("Implementation", &info)?)?.value,
            ),
            _ => return Err(Fault::InvalidArgument("ClientInfo")),
        };
        let supported_versions = args::array(&fields[2])?
            .iter()
            .map(|value| {
                if matches!(value, Value::Integer(0)) {
                    Version::V1
                } else {
                    Version::V2
                }
            })
            .collect();
        let flags = args::record(&fields[3], "Std.Acp.ExperimentalOptions", 7)?;
        let enabled = |index| matches!(flags[index], Value::Boolean(true));
        let mut v2_converter = self.acp_converter(Version::V2)?;
        let v2_caps =
            v2_converter.unpack(&Type::Named("ClientCapabilities".into()), &fields[5], None)?;
        let v2_client_capabilities = fpas_acp::codec::decode(
            v2_converter
                .catalog
                .encode("ClientCapabilities", &v2_caps)?,
        )?
        .value;
        Ok((
            Settings {
                supported_versions,
                client_info,
                client_capabilities,
                v2_client_capabilities,
                experimental: fpas_acp::connection::Experimental {
                    token_usage: enabled(0),
                    providers: enabled(1),
                    mcp: enabled(2),
                    plans: enabled(3),
                    fork: enabled(4),
                    system_prompt: enabled(5),
                    subagents: enabled(6),
                },
            },
            limits,
        ))
    }

    fn acp_encode(
        &self,
        version: Version,
        root: &str,
        payload: &Value,
    ) -> Result<(String, RawJson), Fault> {
        let mut converter = self.acp_converter(version)?;
        let value = converter
            .unpack(&Type::Named(root.into()), payload, None)
            .map_err(|_| Fault::InvalidArgument("Payload"))?;
        let Detached::Enum {
            variant, fields, ..
        } = &value
        else {
            return Err(Fault::InvalidArgument("Payload"));
        };
        let declaration = &converter.catalog.declarations[root];
        let binding::Shape::Enum { variants, .. } = &declaration.shape else {
            return Err(Fault::InvalidArgument("Payload"));
        };
        let key = variants
            .iter()
            .find(|candidate| &candidate.name == variant)
            .and_then(|variant| variant.payload.as_ref())
            .and_then(|ty| {
                if let Type::Named(key) = ty {
                    Some(key)
                } else {
                    None
                }
            })
            .ok_or(Fault::InvalidArgument("Payload"))?;
        let method = if let Some(method) = &converter.catalog.declarations[key].method {
            method.clone()
        } else if let Some(Detached::Record { fields, .. }) = fields.first() {
            if let Some(Detached::String(method)) = fields.first() {
                method.clone()
            } else {
                return Err(Fault::InvalidArgument("Method"));
            }
        } else {
            return Err(Fault::InvalidArgument("Method"));
        };
        Ok((
            method.clone(),
            converter.catalog.encode_message(root, &method, &value)?,
        ))
    }

    fn acp_default_options(&self, location: SourceLocation) -> Result<Value, VmError> {
        let limits = Limits::default();
        let limits = self.layouts.record(
            "Std.Acp.Limits",
            [
                limits.outgoing_requests,
                limits.incoming_requests,
                limits.sessions,
                limits.terminals,
                limits.routes,
                limits.queue_entries,
                limits.outbound_entries,
                limits.queue_bytes,
                limits.message_bytes,
                limits.stderr_bytes,
                limits.shutdown_millis as usize,
                limits.terminal_bytes,
                limits.all_terminal_bytes,
            ]
            .into_iter()
            .map(|value| Value::Integer(value as i64))
            .collect(),
            location,
        )?;
        let experimental = self.layouts.record(
            "Std.Acp.ExperimentalOptions",
            vec![Value::Boolean(false); 7],
            location,
        )?;
        let version = Value::Integer(0);
        let mut converter = self.acp_converter(Version::V1).map_err(|_| {
            crate::vm::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "ACP catalog unavailable",
            )
        })?;
        let capabilities = converter
            .catalog
            .decode(
                "ClientCapabilities",
                &RawJson::parse("{}").map_err(|_| {
                    crate::vm::diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "ACP defaults unavailable",
                    )
                })?,
            )
            .and_then(|value| {
                converter.pack(&Type::Named("ClientCapabilities".into()), value, None)
            })
            .map_err(|_| {
                crate::vm::diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "ACP default capability layout unavailable",
                )
            })?;
        let mut v2_converter = self.acp_converter(Version::V2).map_err(|_| {
            crate::vm::diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "ACP v2 catalog unavailable",
            )
        })?;
        let v2_capabilities = v2_converter
            .catalog
            .decode(
                "ClientCapabilities",
                &RawJson::parse("{}").map_err(|_| {
                    crate::vm::diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "ACP defaults unavailable",
                    )
                })?,
            )
            .and_then(|value| {
                v2_converter.pack(&Type::Named("ClientCapabilities".into()), value, None)
            })
            .map_err(|_| {
                crate::vm::diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "ACP v2 capability layout unavailable",
                )
            })?;
        self.layouts.record(
            "Std.Acp.Options",
            vec![
                Value::OptionNone,
                capabilities,
                Value::Array(vec![version].into()),
                experimental,
                limits,
                v2_capabilities,
            ],
            location,
        )
    }
}
