//! Nominal FPAS values validated before native state is changed.

use crate::Value;
use fpas_mcp::{Fault, HttpOptions, Limits, model::Tool};
use std::time::Duration;

pub(super) fn handle(value: &Value) -> Result<u64, Fault> {
    if let Value::OpaqueHandle(value) = value {
        Ok(*value)
    } else {
        Err(Fault::Closed)
    }
}
pub(super) fn text(value: &Value) -> Result<&str, Fault> {
    if let Value::Str(value) = value {
        Ok(value)
    } else {
        Err(Fault::InvalidArgument("Text"))
    }
}
pub(super) fn integer(value: &Value) -> Result<usize, Fault> {
    if let Value::Integer(value) = value {
        usize::try_from(*value).map_err(|_| Fault::InvalidArgument("Limit"))
    } else {
        Err(Fault::InvalidArgument("Limit"))
    }
}
pub(super) fn optional(value: &Value) -> Result<Option<&Value>, Fault> {
    match value {
        Value::OptionNone => Ok(None),
        Value::OptionSome(v) => Ok(Some(v)),
        _ => Err(Fault::InvalidArgument("Option")),
    }
}
pub(super) fn record<'a>(value: &'a Value, name: &str, count: usize) -> Result<&'a [Value], Fault> {
    if let Value::Record(value) = value {
        let body = value.body();
        if body.layout.type_name.eq_ignore_ascii_case(name) && body.values.len() == count {
            return Ok(&body.values);
        }
    }
    Err(Fault::InvalidArgument("Record"))
}
pub(super) fn raw(value: &Value) -> Result<&str, Fault> {
    let fields = record(value, "Std.Ai.Mcp.Types.RawJson", 1)?;
    let text = text(&fields[0])?;
    if text.len() > 64 * 1024 * 1024 {
        return Err(Fault::LimitExceeded("RawBytes", 64 * 1024 * 1024));
    }
    Ok(text)
}
pub(super) fn json(value: &Value) -> Result<serde_json::Value, Fault> {
    serde_json::from_str(raw(value)?).map_err(|_| Fault::InvalidArgument("Json"))
}
pub(super) fn limits(value: &Value) -> Result<Limits, Fault> {
    let f = record(value, "Std.Ai.Mcp.Limits", 6)?;
    let limits = Limits {
        tools: integer(&f[0])?,
        catalog_bytes: integer(&f[1])?,
        message_bytes: integer(&f[2])?,
        pending_calls: integer(&f[3])?,
        pending_bytes: integer(&f[4])?,
        endpoints: integer(&f[5])?,
    };
    limits.validate()?;
    Ok(limits)
}
pub(super) fn http(value: &Value) -> Result<HttpOptions, Fault> {
    let f = record(value, "Std.Ai.Mcp.HttpOptions", 4)?;
    Ok(HttpOptions {
        address: text(&f[0])?
            .parse()
            .map_err(|_| Fault::InvalidArgument("Address"))?,
        connections: integer(&f[1])?,
        request_timeout: Duration::from_millis(integer(&f[2])? as u64),
        shutdown_timeout: Duration::from_millis(integer(&f[3])? as u64),
    })
}
fn object(value: &Value) -> Result<fpas_mcp::model::JsonObject, Fault> {
    json(value)?
        .as_object()
        .cloned()
        .ok_or(Fault::InvalidSchema("Object"))
}
pub(super) fn definition(value: &Value) -> Result<Tool, Fault> {
    let f = record(value, "Std.Ai.Mcp.Types.ToolDefinition", 6)?;
    let mut tool = Tool::new(
        text(&f[0])?.to_owned(),
        text(&f[1])?.to_owned(),
        object(&f[2])?,
    );
    tool.output_schema = optional(&f[3])?
        .map(object)
        .transpose()?
        .map(std::sync::Arc::new);
    tool.annotations = optional(&f[4])?
        .map(|v| {
            serde_json::from_value(json(v)?).map_err(|_| Fault::InvalidArgument("Annotations"))
        })
        .transpose()?;
    tool.meta = optional(&f[5])?
        .map(|v| serde_json::from_value(json(v)?).map_err(|_| Fault::InvalidArgument("Meta")))
        .transpose()?;
    Ok(tool)
}
