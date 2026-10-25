//! Stack-safe formatting for runtime values.

use std::fmt;

use super::{EnumValue, Value};

enum DisplayPart<'a> {
    Value(&'a Value, bool),
    Text(&'a str),
    Error(&'a EnumValue),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pending = vec![DisplayPart::Value(self, false)];
        while let Some(part) = pending.pop() {
            let (value, field) = match part {
                DisplayPart::Value(value, field) => (value, field),
                DisplayPart::Error(body) => {
                    error_frame(formatter, &mut pending, body)?;
                    continue;
                }
                DisplayPart::Text(text) => {
                    formatter.write_str(text)?;
                    continue;
                }
            };

            match value {
                Value::Integer(number) => write!(formatter, "{number}")?,
                Value::Real(number) => write!(formatter, "{number}")?,
                Value::Boolean(boolean) => write!(formatter, "{boolean}")?,
                Value::Str(string) if field => quoted(formatter, string)?,
                Value::Str(string) => write!(formatter, "{string}")?,
                Value::Enum(value) => {
                    let body = value.body();
                    if body.layout.error.is_some() && !field {
                        pending.push(DisplayPart::Error(body));
                        continue;
                    }
                    write!(
                        formatter,
                        "{}.{}",
                        body.layout.type_name, body.layout.variant
                    )?;
                    if !body.values.is_empty() {
                        formatter.write_str("(")?;
                        pending.push(DisplayPart::Text(")"));
                        push_values(&mut pending, &body.values, field);
                    }
                }
                Value::Array(values) => {
                    formatter.write_str("[")?;
                    pending.push(DisplayPart::Text("]"));
                    push_values(&mut pending, values, field);
                }
                Value::Dict(pairs) => {
                    formatter.write_str("{")?;
                    pending.push(DisplayPart::Text("}"));
                    for (index, (key, value)) in pairs.iter().enumerate().rev() {
                        pending.push(DisplayPart::Value(value, field));
                        pending.push(DisplayPart::Text(": "));
                        pending.push(DisplayPart::Value(key, field));
                        if index > 0 {
                            pending.push(DisplayPart::Text(", "));
                        }
                    }
                }
                Value::Record(record) => {
                    let body = record.body();
                    write!(formatter, "{}{{", body.layout.type_name)?;
                    pending.push(DisplayPart::Text("}"));
                    let field_count = body.layout.fields.len().min(body.values.len());
                    for index in (0..field_count).rev() {
                        pending.push(DisplayPart::Value(&body.values[index], field));
                        pending.push(DisplayPart::Text(": "));
                        pending.push(DisplayPart::Text(&body.layout.fields[index]));
                        if index > 0 {
                            pending.push(DisplayPart::Text(", "));
                        }
                    }
                }
                Value::Unit => formatter.write_str("()")?,
                Value::ResultOk(value) => {
                    formatter.write_str("Ok(")?;
                    pending.push(DisplayPart::Text(")"));
                    pending.push(DisplayPart::Value(value, field));
                }
                Value::ResultError(value) => {
                    formatter.write_str("Error(")?;
                    pending.push(DisplayPart::Text(")"));
                    pending.push(DisplayPart::Value(value, field));
                }
                Value::OptionSome(value) => {
                    formatter.write_str("Some(")?;
                    pending.push(DisplayPart::Text(")"));
                    pending.push(DisplayPart::Value(value, field));
                }
                Value::OptionNone => formatter.write_str("None")?,
                Value::Function(function) => {
                    write!(formatter, "<function {}>", function.name)?;
                }
                Value::Cell(_) => formatter.write_str("<cell>")?,
                Value::Reference(_) => formatter.write_str("<reference>")?,
                Value::Task(id) => write!(formatter, "<task {id}>")?,
                Value::OpaqueHandle(_) => formatter.write_str("<opaque handle>")?,
            }
        }
        Ok(())
    }
}

fn push_values<'a>(pending: &mut Vec<DisplayPart<'a>>, values: &'a [Value], field: bool) {
    for (index, value) in values.iter().enumerate().rev() {
        pending.push(DisplayPart::Value(value, field));
        if index > 0 {
            pending.push(DisplayPart::Text(", "));
        }
    }
}

fn error_frame<'a>(
    formatter: &mut fmt::Formatter<'_>,
    pending: &mut Vec<DisplayPart<'a>>,
    body: &'a EnumValue,
) -> fmt::Result {
    write!(
        formatter,
        "{}.{}",
        body.layout.type_name, body.layout.variant
    )?;
    let Some(error) = &body.layout.error else {
        return Ok(());
    };
    if let Some(message) = &error.message {
        formatter.write_str(": ")?;
        escaped(formatter, message, false)?;
    }
    if error.hide_payload {
        return Ok(());
    }
    if let Some(cause) = &error.cause {
        match body.values.get(usize::from(cause.field)) {
            Some(value) => pending.push(DisplayPart::Value(value, false)),
            None => pending.push(DisplayPart::Text("<invalid Cause>")),
        }
        pending.push(DisplayPart::Text("\ncaused by "));
    }
    for (index, (name, value)) in body
        .layout
        .fields
        .iter()
        .zip(&body.values)
        .enumerate()
        .rev()
    {
        if error
            .cause
            .as_ref()
            .is_some_and(|cause| usize::from(cause.field) == index)
        {
            continue;
        }
        pending.push(DisplayPart::Value(value, true));
        pending.push(DisplayPart::Text(" = "));
        pending.push(DisplayPart::Text(name));
        pending.push(DisplayPart::Text("\n  "));
    }
    Ok(())
}

fn quoted(formatter: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    formatter.write_str("\"")?;
    escaped(formatter, text, true)?;
    formatter.write_str("\"")
}

fn escaped(formatter: &mut fmt::Formatter<'_>, text: &str, json: bool) -> fmt::Result {
    for ch in text.chars() {
        match ch {
            '"' if json => formatter.write_str("\\\"")?,
            '\\' if json => formatter.write_str("\\\\")?,
            '\n' => formatter.write_str("\\n")?,
            '\r' => formatter.write_str("\\r")?,
            '\t' => formatter.write_str("\\t")?,
            '\u{08}' => formatter.write_str("\\b")?,
            '\u{0c}' => formatter.write_str("\\f")?,
            ch if ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}') => {
                write!(formatter, "\\u{:04x}", u32::from(ch))?
            }
            ch => write!(formatter, "{ch}")?,
        }
    }
    Ok(())
}
