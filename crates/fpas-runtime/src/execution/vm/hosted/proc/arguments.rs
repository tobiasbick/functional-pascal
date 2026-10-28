//! Decode admitted process records without copying their potentially large input text.

use std::time::Duration;

use crate::Value;

use super::failure::Failure;
use super::options::{MAX_IO, Options, Request};

pub(super) fn command_context(values: &[Value]) -> &str {
    match values.first() {
        Some(Value::Str(text)) => text,
        _ => "",
    }
}

pub(super) fn parse(
    values: &[Value],
    options: bool,
    capture: bool,
) -> Result<Request<'_>, Failure> {
    if values.len() != if options { 3 } else { 2 } {
        return Err(Failure::argument(
            "Arguments",
            "Incorrect intrinsic argument count",
        ));
    }
    let command = text(&values[0], "Command")?;
    let Value::Array(arguments) = &values[1] else {
        return Err(Failure::argument("Args", "Expected an array of strings"));
    };
    if arguments.len() > 4096 {
        return Err(Failure::limit("arguments", 4096));
    }
    let arguments = arguments
        .iter()
        .map(|value| text(value, "Args"))
        .collect::<Result<Vec<_>, _>>()?;
    let request = Request {
        command,
        arguments,
        options: if options {
            parse_options(&values[2])?
        } else {
            Options::default()
        },
        capture,
        inherit_input: !options && !capture,
    };
    request.validate()?;
    Ok(request)
}

pub(super) fn parse_start(values: &[Value]) -> Result<Request<'_>, Failure> {
    if values.len() != 3 {
        return Err(Failure::argument(
            "Arguments",
            "Expected command, arguments and start options",
        ));
    }
    let mut request = parse(&values[..2], false, true)?;
    request.options = configuration(record(&values[2], "Std.Os.Proc.ProcessStartOptions", 2)?)?;
    request.validate()?;
    Ok(request)
}

pub(super) fn parse_interactive(values: &[Value]) -> Result<Request<'_>, Failure> {
    if values.len() != 5 {
        return Err(Failure::argument(
            "Arguments",
            "Expected command, arguments, start options, timeout and cancellation token",
        ));
    }
    let mut request = parse_start(&values[..3])?;
    request.capture = false;
    request.inherit_input = true;
    request.options.timeout = timeout(&values[3])?;
    request.options.token = token(&values[4])?;
    Ok(request)
}

fn configuration(fields: &[Value]) -> Result<Options<'_>, Failure> {
    require_option(&fields[0])?;
    let directory = optional(&fields[0])
        .map(|value| text(value, "WorkingDirectory"))
        .transpose()?;
    let Value::Array(environment) = &fields[1] else {
        return Err(Failure::argument("Environment", "Expected an array"));
    };
    if environment.len() > 1024 {
        return Err(Failure::limit("environment", 1024));
    }
    let mut entries = Vec::with_capacity(environment.len());
    for value in environment {
        let fields = record(value, "Std.Os.Proc.EnvironmentVariable", 2)?;
        require_option(&fields[1])?;
        entries.push((
            text(&fields[0], "Environment")?,
            optional(&fields[1])
                .map(|value| text(value, "Environment"))
                .transpose()?,
        ));
    }
    Ok(Options {
        directory,
        environment: entries,
        ..Options::default()
    })
}

fn parse_options(value: &Value) -> Result<Options<'_>, Failure> {
    let fields = record(value, "Std.Os.Proc.ProcessOptions", 6)?;
    let configuration = configuration(fields)?;
    let input = text(&fields[2], "Input")?;
    let timeout = timeout(&fields[3])?;
    let maximum = usize::try_from(integer(&fields[4], "MaxOutputBytes")?)
        .ok()
        .filter(|value| *value <= MAX_IO)
        .ok_or_else(|| {
            Failure::argument("MaxOutputBytes", "Output limit must be in 0..67108864")
        })?;
    let token = token(&fields[5])?;
    Ok(Options {
        input,
        timeout,
        maximum,
        token,
        ..configuration
    })
}

fn timeout(value: &Value) -> Result<Option<Duration>, Failure> {
    require_option(value)?;
    optional(value)
        .map(|value| {
            let millis = integer(value, "TimeoutMillis")?;
            u64::try_from(millis)
                .map(Duration::from_millis)
                .map_err(|_| Failure::argument("TimeoutMillis", "Timeout must not be negative"))
        })
        .transpose()
}

fn token(value: &Value) -> Result<Option<u64>, Failure> {
    require_option(value)?;
    optional(value)
        .map(|value| match value {
            Value::OpaqueHandle(token) => Ok(*token),
            _ => Err(Failure::argument(
                "CancellationToken",
                "Expected a cancellation token",
            )),
        })
        .transpose()
}

fn optional(value: &Value) -> Option<&Value> {
    if let Value::OptionSome(value) = value {
        Some(value.as_ref())
    } else {
        None
    }
}

fn require_option(value: &Value) -> Result<(), Failure> {
    if matches!(value, Value::OptionNone | Value::OptionSome(_)) {
        Ok(())
    } else {
        Err(Failure::argument("Options", "Expected an Option value"))
    }
}

fn record<'a>(value: &'a Value, nominal: &str, length: usize) -> Result<&'a [Value], Failure> {
    if let Value::Record(record) = value
        && record.body().layout.type_name == nominal
        && record.body().values.len() == length
    {
        return Ok(&record.body().values);
    }
    Err(Failure::argument("Options", format!("Expected {nominal}")))
}

fn text<'a>(value: &'a Value, name: &'static str) -> Result<&'a str, Failure> {
    if let Value::Str(text) = value {
        Ok(text)
    } else {
        Err(Failure::argument(name, "Expected string"))
    }
}

pub(super) fn integer(value: &Value, name: &'static str) -> Result<i64, Failure> {
    if let Value::Integer(number) = value {
        Ok(*number)
    } else {
        Err(Failure::argument(name, "Expected integer"))
    }
}
