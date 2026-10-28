//! Validate detached FPAS configuration before allocating handles or starting children.

use super::super::proc::AcpLaunch;
use crate::Value;
use fpas_acp::{
    RawJson,
    connection::{Fault, Limits},
    schema::Version,
};

pub(super) fn handle(value: &Value) -> Result<u64, Fault> {
    if let Value::OpaqueHandle(id) = value {
        Ok(*id)
    } else {
        Err(Fault::Closed)
    }
}
pub(super) fn text(value: &Value) -> Result<&str, Fault> {
    if let Value::Str(text) = value {
        Ok(text)
    } else {
        Err(Fault::InvalidArgument("Text"))
    }
}
pub(super) fn record<'a>(
    value: &'a Value,
    name: &str,
    length: usize,
) -> Result<&'a [Value], Fault> {
    if let Value::Record(value) = value {
        let body = value.body();
        if body.layout.type_name.eq_ignore_ascii_case(name) && body.values.len() == length {
            return Ok(&body.values);
        }
    }
    Err(Fault::InvalidArgument("Record"))
}
pub(super) fn optional(value: &Value) -> Result<Option<&Value>, Fault> {
    match value {
        Value::OptionNone => Ok(None),
        Value::OptionSome(value) => Ok(Some(value)),
        _ => Err(Fault::InvalidArgument("Option")),
    }
}
pub(super) fn array(value: &Value) -> Result<&[Value], Fault> {
    if let Value::Array(value) = value {
        Ok(value)
    } else {
        Err(Fault::InvalidArgument("Array"))
    }
}
pub(super) fn raw(value: &Value) -> Result<RawJson, Fault> {
    RawJson::parse(text(&record(value, "Std.Ai.Acp.Types.RawJson", 1)?[0])?).map_err(Fault::from)
}
pub(super) fn unsigned(value: &Value) -> Result<usize, Fault> {
    if let Value::Integer(value) = value {
        usize::try_from(*value).map_err(|_| Fault::InvalidArgument("Limits"))
    } else {
        Err(Fault::InvalidArgument("Limits"))
    }
}
pub(super) fn timeout(value: &Value) -> Result<Option<u64>, Fault> {
    optional(value)?
        .map(|value| match value {
            Value::Integer(value) if *value >= 0 => Ok(*value as u64),
            _ => Err(Fault::InvalidArgument("TimeoutMillis")),
        })
        .transpose()
}

pub(super) fn options(value: &Value) -> Result<(Version, Limits), Fault> {
    let fields = record(value, "Std.Ai.Acp.Options", 6)?;
    let versions = array(&fields[2])?;
    if versions.is_empty() || versions.len() > 2 {
        return Err(Fault::InvalidArgument("SupportedVersions"));
    }
    let mut selected = Version::V1;
    let mut seen = [false, false];
    for version in versions {
        let index = match version {
            Value::Integer(0) => 0,
            Value::Integer(1) => {
                selected = Version::V2;
                1
            }
            _ => return Err(Fault::InvalidArgument("SupportedVersions")),
        };
        if seen[index] {
            return Err(Fault::InvalidArgument("SupportedVersions"));
        }
        seen[index] = true;
    }
    let experimental = record(&fields[3], "Std.Ai.Acp.ExperimentalOptions", 7)?;
    if experimental
        .iter()
        .any(|value| !matches!(value, Value::Boolean(_)))
    {
        return Err(Fault::InvalidArgument("ExperimentalOptions"));
    }
    let fields = record(&fields[4], "Std.Ai.Acp.Limits", 13)?
        .iter()
        .map(unsigned)
        .collect::<Result<Vec<_>, _>>()?;
    let limits = Limits {
        outgoing_requests: fields[0],
        incoming_requests: fields[1],
        sessions: fields[2],
        terminals: fields[3],
        routes: fields[4],
        queue_entries: fields[5],
        outbound_entries: fields[6],
        queue_bytes: fields[7],
        message_bytes: fields[8],
        stderr_bytes: fields[9],
        shutdown_millis: fields[10] as u64,
        terminal_bytes: fields[11],
        all_terminal_bytes: fields[12],
    };
    limits.validate()?;
    Ok((selected, limits))
}

pub(super) fn launch(value: &Value) -> Result<AcpLaunch, Fault> {
    let fields = record(value, "Std.Ai.Acp.Launch", 4)?;
    let arguments = array(&fields[1])?;
    let environment = array(&fields[3])?;
    if arguments.len() > 4096 {
        return Err(Fault::LimitExceeded("Arguments", 4096));
    }
    if environment.len() > 1024 {
        return Err(Fault::LimitExceeded("Environment", 1024));
    }
    Ok(AcpLaunch {
        command: text(&fields[0])?.into(),
        arguments: arguments
            .iter()
            .map(|value| text(value).map(str::to_owned))
            .collect::<Result<_, _>>()?,
        directory: optional(&fields[2])?
            .map(text)
            .transpose()?
            .map(str::to_owned),
        environment: environment
            .iter()
            .map(|value| {
                let fields = record(value, "Std.Ai.Acp.EnvironmentVariable", 2)?;
                Ok((
                    text(&fields[0])?.into(),
                    optional(&fields[1])?
                        .map(text)
                        .transpose()?
                        .map(str::to_owned),
                ))
            })
            .collect::<Result<_, Fault>>()?,
    })
}
