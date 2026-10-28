//! Shared FPAS handler ownership and cooperative MCP execution.

mod acp;
mod arguments;
mod registry;
mod values;
pub(in crate::execution::vm) mod wait;
pub(in crate::execution::vm) use registry::Registry;

use crate::execution::vm::{VmError, worker::Worker};
use crate::stdlib::AggregateFactory;
use crate::{Intrinsic, McpIntrinsic as Op, RootKind, SourceLocation, Value};
use fpas_mcp::{Fault, HttpError, model::CallToolResult};

pub(super) enum Failure {
    Core(Fault),
    Transport(HttpError),
    Cancelled,
}
impl From<Fault> for Failure {
    fn from(value: Fault) -> Self {
        Self::Core(value)
    }
}
impl From<HttpError> for Failure {
    fn from(value: HttpError) -> Self {
        Self::Transport(value)
    }
}

impl Worker {
    pub(super) fn execute_mcp_intrinsic(
        &self,
        intrinsic: Intrinsic,
        args: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Mcp(op) = intrinsic else {
            return Ok(None);
        };
        let value = match (op, args) {
            (Op::DefaultLimits, []) => {
                let v = fpas_mcp::Limits::default();
                self.layouts.record(
                    "Std.Ai.Mcp.Limits",
                    [
                        v.tools,
                        v.catalog_bytes,
                        v.message_bytes,
                        v.pending_calls,
                        v.pending_bytes,
                        v.endpoints,
                    ]
                    .into_iter()
                    .map(|v| Value::Integer(v as i64))
                    .collect(),
                    location,
                )?
            }
            (Op::DefaultHttpOptions, []) => {
                let v = fpas_mcp::HttpOptions::default();
                self.layouts.record(
                    "Std.Ai.Mcp.HttpOptions",
                    vec![
                        Value::Str(v.address.to_string().into()),
                        Value::Integer(v.connections as i64),
                        Value::Integer(v.request_timeout.as_millis() as i64),
                        Value::Integer(v.shutdown_timeout.as_millis() as i64),
                    ],
                    location,
                )?
            }
            (Op::TextResult | Op::ErrorResult, [Value::Str(text)]) => {
                let content = vec![fpas_mcp::model::ContentBlock::text(text.to_string())];
                let result = if op == Op::ErrorResult {
                    CallToolResult::error(content)
                } else {
                    CallToolResult::success(content)
                };
                self.mcp_tool_result(&result, location)?
            }
            (Op::IsCancelled, [context]) => {
                let cancelled =
                    (|| {
                        let f = arguments::record(context, "Std.Ai.Mcp.Types.RequestContext", 8)?;
                        Ok::<_, Fault>(self.hosted.mcp.is_cancelled(
                            arguments::handle(&f[0])?,
                            arguments::integer(&f[2])? as u64,
                        ))
                    })()
                    .unwrap_or(true);
                Value::Boolean(cancelled)
            }
            _ => {
                if let (Op::RegisterTool, [_, _, handler]) = (op, args) {
                    self.validate_callback(handler, 2)?;
                }
                let result = (|| -> Result<Value, Failure> {
                    match (op, args) {
                        (Op::Create, [limits]) => Ok(Value::OpaqueHandle(
                            self.hosted.mcp.create(arguments::limits(limits)?)?,
                        )),
                        (Op::RegisterTool, [server, definition, handler]) => {
                            let roots = self
                                .domain
                                .root_values(RootKind::Native, vec![handler.clone()]);
                            self.hosted.mcp.register(
                                arguments::handle(server)?,
                                arguments::definition(definition)?,
                                roots,
                            )?;
                            Ok(Value::Boolean(true))
                        }
                        (Op::Access, [endpoint]) => {
                            let access =
                                self.hosted.mcp.publication(arguments::handle(endpoint)?)?;
                            let header = self
                                .layouts
                                .record(
                                    "Std.Ai.Mcp.Types.HttpHeader",
                                    vec![
                                        Value::Str("Authorization".into()),
                                        Value::Str(access.authorization().into()),
                                    ],
                                    location,
                                )
                                .map_err(|_| Fault::InvalidResult)?;
                            self.layouts
                                .record(
                                    "Std.Ai.Mcp.Types.HttpAccess",
                                    vec![
                                        Value::Str(access.url().into()),
                                        Value::Array(vec![header].into()),
                                    ],
                                    location,
                                )
                                .map_err(|_| Fault::InvalidResult)
                                .map_err(Into::into)
                        }
                        (Op::ParseJson, [Value::Str(text)]) => {
                            if text.len() > 64 * 1024 * 1024 {
                                return Err(
                                    Fault::LimitExceeded("RawBytes", 64 * 1024 * 1024).into()
                                );
                            }
                            serde_json::from_str::<serde_json::Value>(text)
                                .map_err(|_| Fault::InvalidArgument("Json"))?;
                            self.mcp_raw(text, location)
                                .map_err(|_| Fault::InvalidResult.into())
                        }
                        (Op::JsonMember, [raw, Value::Str(name)]) => {
                            let json = arguments::json(raw)?;
                            let object =
                                json.as_object().ok_or(Fault::InvalidArgument("Object"))?;
                            object
                                .get(&**name)
                                .map(|v| {
                                    self.mcp_raw(&v.to_string(), location)
                                        .map(Value::option_some)
                                })
                                .transpose()
                                .map(|v| v.unwrap_or(Value::OptionNone))
                                .map_err(|_| Fault::InvalidResult.into())
                        }
                        (Op::ParseToolResult, [raw]) => {
                            let result =
                                serde_json::from_value::<CallToolResult>(arguments::json(raw)?)
                                    .map_err(|_| Fault::InvalidResult)?;
                            self.mcp_tool_result(&result, location)
                                .map_err(|_| Fault::InvalidResult.into())
                        }
                        (Op::StructuredResult, [raw]) => {
                            let json = arguments::json(raw)?;
                            if !json.is_object() {
                                return Err(Fault::InvalidResult.into());
                            }
                            self.mcp_tool_result(&CallToolResult::structured(json), location)
                                .map_err(|_| Fault::InvalidResult.into())
                        }
                        _ => Err(Fault::InvalidArgument("Arguments").into()),
                    }
                })();
                self.mcp_result(result, location)?
            }
        };
        Ok(Some(Some(value)))
    }
}
