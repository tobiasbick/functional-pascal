//! Decode admitted process records without copying their potentially large input text.

use std::time::Duration;

use fpas_bytecode::Value;

use super::failure::Failure;
use super::options::{MAX_IO, Options, Request};

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

fn parse_options(value: &Value) -> Result<Options<'_>, Failure> {
    let fields = record(value, "Std.Proc.ProcessOptions", 6)?;
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
        let fields = record(value, "Std.Proc.EnvironmentVariable", 2)?;
        entries.push((
            text(&fields[0], "Environment")?,
            optional(&fields[1])
                .map(|value| text(value, "Environment"))
                .transpose()?,
        ));
    }
    let input = text(&fields[2], "Input")?;
    let timeout = optional(&fields[3])
        .map(|value| {
            let millis = integer(value, "TimeoutMillis")?;
            u64::try_from(millis)
                .map(Duration::from_millis)
                .map_err(|_| Failure::argument("TimeoutMillis", "Timeout must not be negative"))
        })
        .transpose()?;
    let maximum = usize::try_from(integer(&fields[4], "MaxOutputBytes")?)
        .ok()
        .filter(|value| *value <= MAX_IO)
        .ok_or_else(|| {
            Failure::argument("MaxOutputBytes", "Output limit must be in 0..67108864")
        })?;
    let token = optional(&fields[5])
        .map(|value| match value {
            Value::OpaqueHandle(token) => Ok(*token),
            _ => Err(Failure::argument(
                "CancellationToken",
                "Expected a cancellation token",
            )),
        })
        .transpose()?;
    // Optional fields are checked separately so malformed wrappers never become defaults.
    for index in [0, 3, 5] {
        require_option(&fields[index])?;
    }
    for value in environment {
        require_option(&record(value, "Std.Proc.EnvironmentVariable", 2)?[1])?;
    }
    Ok(Options {
        directory,
        environment: entries,
        input,
        timeout,
        maximum,
        token,
    })
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

fn integer(value: &Value, name: &'static str) -> Result<i64, Failure> {
    if let Value::Integer(number) = value {
        Ok(*number)
    } else {
        Err(Failure::argument(name, "Expected integer"))
    }
}
