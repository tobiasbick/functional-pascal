//! Checked, detached ACP values; no process or connection effects.

use crate::{AcpIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_acp_model::{CodecError, ErrorKind, RawJson, UInt64};

use crate::stdlib::{
    StdError,
    intrinsic_args::{IntrinsicCall, pop_value},
};

const RAW: &str = "Std.Ai.Acp.Types.RawJson";
const UNSIGNED: &str = "Std.Ai.Acp.Types.UInt64";

pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let Intrinsic::Acp(operation) = intrinsic else {
        return Ok(None);
    };
    if operation.codec().is_some() {
        return Ok(None);
    }
    if operation == AcpIntrinsic::RawJsonMember {
        let name = pop_value(call, location)?.clone();
        let value = pop_value(call, location)?;
        let member = (|| {
            let name = text(&name)?;
            let raw = record_text(value, RAW, "Text").and_then(RawJson::parse)?;
            raw.member(name)
        })();
        let result = match member {
            Ok(Some(raw)) => Value::result_ok(Value::option_some(call.factory().record(
                RAW,
                vec![Value::Str(raw.text().into())],
                location,
            )?)),
            Ok(None) => Value::result_ok(Value::OptionNone),
            Err(error) => Value::result_error(error_value(call, error, location)?),
        };
        call.push(result);
        return Ok(Some(()));
    }
    let argument = pop_value(call, location)?;
    let result = match operation {
        AcpIntrinsic::ParseRawJson => text(argument)
            .and_then(RawJson::parse)
            .map(|raw| (RAW, raw.text().to_owned()))
            .map(|(name, text)| (Some(name), Value::Str(text.into()))),
        AcpIntrinsic::RawJsonText => record_text(argument, RAW, "Text")
            .and_then(RawJson::parse)
            .map(|raw| (None, Value::Str(raw.text().into()))),
        AcpIntrinsic::ParseUInt64 => text(argument)
            .and_then(UInt64::parse)
            .map(|value| (Some(UNSIGNED), Value::Str(value.decimal().into()))),
        AcpIntrinsic::UInt64Text => {
            unsigned(argument).map(|value| (None, Value::Str(value.decimal().into())))
        }
        AcpIntrinsic::UInt64FromInteger => match argument {
            Value::Integer(value) => UInt64::from_integer(*value)
                .map(|value| (Some(UNSIGNED), Value::Str(value.decimal().into()))),
            _ => Err(CodecError::new(ErrorKind::InvalidInteger)),
        },
        AcpIntrinsic::UInt64ToInteger => unsigned(argument)
            .and_then(UInt64::to_integer)
            .map(|value| (None, Value::Integer(value))),
        _ => return Ok(None),
    };
    let result = match result {
        Ok((record, value)) => Value::result_ok(if let Some(name) = record {
            call.factory().record(name, vec![value], location)?
        } else {
            value
        }),
        Err(error) => Value::result_error(error_value(call, error, location)?),
    };
    call.push(result);
    Ok(Some(()))
}

fn text(value: &Value) -> Result<&str, CodecError> {
    match value {
        Value::Str(text) => Ok(text),
        _ => Err(CodecError::new(ErrorKind::InvalidMessage)),
    }
}

fn record_text<'a>(value: &'a Value, name: &str, field: &str) -> Result<&'a str, CodecError> {
    if let Value::Record(record) = value {
        let record = record.body();
        if record.layout.type_name.eq_ignore_ascii_case(name)
            && record.layout.fields.len() == 1
            && record.layout.fields[0].eq_ignore_ascii_case(field)
            && let [value] = record.values.as_slice()
        {
            return text(value);
        }
    }
    Err(CodecError::new(ErrorKind::InvalidMessage))
}

fn unsigned(value: &Value) -> Result<UInt64, CodecError> {
    record_text(value, UNSIGNED, "Digits").and_then(UInt64::canonical)
}

fn error_value(
    call: &IntrinsicCall<'_>,
    error: CodecError,
    location: SourceLocation,
) -> Result<Value, StdError> {
    let (variant, fields) = match error.kind {
        ErrorKind::LimitExceeded => (
            "LimitExceeded",
            vec![
                Value::Str("RawJsonBytes".into()),
                Value::Integer(fpas_acp_model::raw::MAX_RAW_BYTES as i64),
            ],
        ),
        ErrorKind::InvalidInteger => (
            "InvalidArgument",
            vec![
                Value::Str("UInt64".into()),
                Value::Str(
                    "Expected a canonical unsigned decimal value in the target range".into(),
                ),
            ],
        ),
        _ => {
            let position = |value: Option<usize>| {
                value
                    .and_then(|value| i64::try_from(value).ok())
                    .map_or(Value::OptionNone, |value| {
                        Value::option_some(Value::Integer(value))
                    })
            };
            (
                "InvalidMessage",
                vec![
                    position(error.line),
                    position(error.column),
                    Value::Str(format!("{:?}", error.kind).into()),
                ],
            )
        }
    };
    call.enumeration("Std.Ai.Acp.Types.AcpError", variant, fields, location)
}
