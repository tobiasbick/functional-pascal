//! Typed protocol value conversion, without process or connection state.

mod arguments;
mod client;
mod dispatch;
mod host;
mod outcomes;
mod registry;
#[cfg(test)]
mod tests;
mod values;
pub(in crate::execution::vm) mod wait;
pub(in crate::execution::vm) use registry::Registry;

use crate::execution::vm::{VmError, worker::Worker};
use crate::stdlib::AggregateFactory;
use crate::{Intrinsic, SourceLocation, Value};
use fpas_acp::binding::CatalogCodec as _;
use fpas_acp::{
    CodecError, ErrorKind, RawJson,
    binding::{self, Type},
    schema::Version,
};

impl Worker {
    pub(super) fn execute_acp_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Acp(operation) = intrinsic else {
            return Ok(None);
        };
        if matches!(
            operation,
            crate::AcpIntrinsic::RegisterTool | crate::AcpIntrinsic::AttachMcpServer
        ) {
            return self
                .execute_acp_tools(operation, arguments, location)
                .map(|v| Some(Some(v)));
        }
        if operation.connection() {
            return self
                .execute_acp_connection(operation, arguments, location)
                .map(|value| Some(Some(value)));
        }
        let Some((version, decode, root)) = operation.codec() else {
            return Ok(None);
        };
        let version = if version == 2 {
            Version::V2
        } else {
            Version::V1
        };
        let catalog = binding::catalog(version);
        let result: Result<(Option<(String, RawJson)>, Value), CodecError> = (|| {
            let catalog = catalog?;
            let [Value::Str(method), payload] = arguments else {
                return Err(CodecError::new(ErrorKind::InvalidMessage));
            };
            let mut converter = values::Converter {
                catalog,
                image: self.executable.executable(),
                layouts: &self.layouts,
                depth: 0,
            };
            if decode {
                let raw = RawJson::parse(converter.record_text(
                    payload,
                    "Std.Ai.Acp.Types.RawJson",
                    "Text",
                    None,
                )?)?;
                let decoded = catalog.decode_message(root, method, raw)?;
                let payload = converter.pack(&Type::Named(root.into()), decoded.payload, None)?;
                Ok((
                    Some((
                        format!(
                            "{}.Decoded{root}",
                            if version == Version::V1 {
                                "Std.Ai.Acp.Types"
                            } else {
                                "Std.Ai.Acp.V2"
                            }
                        ),
                        decoded.raw,
                    )),
                    payload,
                ))
            } else {
                let payload = converter.unpack(&Type::Named(root.into()), payload, None)?;
                let raw = catalog.encode_message(root, method, &payload)?;
                Ok((None, Value::Str(raw.text().into())))
            }
        })();
        let result = match result {
            Ok((Some((name, raw)), payload)) => {
                let raw = self.layouts.record(
                    "Std.Ai.Acp.Types.RawJson",
                    vec![Value::Str(raw.text().into())],
                    location,
                )?;
                Value::result_ok(self.layouts.record(&name, vec![payload, raw], location)?)
            }
            Ok((None, text)) => Value::result_ok(self.layouts.record(
                "Std.Ai.Acp.Types.RawJson",
                vec![text],
                location,
            )?),
            Err(error) => {
                let position = |value: Option<usize>| {
                    value
                        .and_then(|value| i64::try_from(value).ok())
                        .map_or(Value::OptionNone, |value| {
                            Value::option_some(Value::Integer(value))
                        })
                };
                let (variant, fields) = if error.kind == ErrorKind::LimitExceeded {
                    (
                        "LimitExceeded",
                        vec![
                            Value::Str("PayloadBytes".into()),
                            Value::Integer(fpas_acp::raw::MAX_RAW_BYTES as i64),
                        ],
                    )
                } else {
                    (
                        "InvalidMessage",
                        vec![
                            position(error.line),
                            position(error.column),
                            Value::Str(format!("{:?}", error.kind).into()),
                        ],
                    )
                };
                Value::result_error(self.layouts.enumeration(
                    "Std.Ai.Acp.Types.AcpError",
                    variant,
                    fields,
                    location,
                )?)
            }
        };
        Ok(Some(Some(result)))
    }
}
