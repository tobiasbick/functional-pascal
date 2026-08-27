//! Stack-safe formatting for runtime values.

use std::fmt;

use super::Value;

enum DisplayPart<'a> {
    Value(&'a Value),
    Text(&'a str),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pending = vec![DisplayPart::Value(self)];
        while let Some(part) = pending.pop() {
            let value = match part {
                DisplayPart::Value(value) => value,
                DisplayPart::Text(text) => {
                    formatter.write_str(text)?;
                    continue;
                }
            };

            match value {
                Value::Integer(number) => write!(formatter, "{number}")?,
                Value::Real(number) => write!(formatter, "{number}")?,
                Value::Boolean(boolean) => write!(formatter, "{boolean}")?,
                Value::Str(string) => write!(formatter, "{string}")?,
                Value::Enum(value) => {
                    let body = value.body();
                    write!(
                        formatter,
                        "{}.{}",
                        body.layout.type_name, body.layout.variant
                    )?;
                    if !body.values.is_empty() {
                        formatter.write_str("(")?;
                        pending.push(DisplayPart::Text(")"));
                        push_values(&mut pending, &body.values);
                    }
                }
                Value::Array(values) => {
                    formatter.write_str("[")?;
                    pending.push(DisplayPart::Text("]"));
                    push_values(&mut pending, values);
                }
                Value::Dict(pairs) => {
                    formatter.write_str("{")?;
                    pending.push(DisplayPart::Text("}"));
                    for (index, (key, value)) in pairs.iter().enumerate().rev() {
                        pending.push(DisplayPart::Value(value));
                        pending.push(DisplayPart::Text(": "));
                        pending.push(DisplayPart::Value(key));
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
                        pending.push(DisplayPart::Value(&body.values[index]));
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
                    pending.push(DisplayPart::Value(value));
                }
                Value::ResultError(value) => {
                    formatter.write_str("Error(")?;
                    pending.push(DisplayPart::Text(")"));
                    pending.push(DisplayPart::Value(value));
                }
                Value::OptionSome(value) => {
                    formatter.write_str("Some(")?;
                    pending.push(DisplayPart::Text(")"));
                    pending.push(DisplayPart::Value(value));
                }
                Value::OptionNone => formatter.write_str("None")?,
                Value::Function(function) => {
                    write!(formatter, "<function {}>", function.name)?;
                }
                Value::Cell(_) => formatter.write_str("<cell>")?,
                Value::Task(id) => write!(formatter, "<task {id}>")?,
                Value::OpaqueHandle(_) => formatter.write_str("<opaque handle>")?,
            }
        }
        Ok(())
    }
}

fn push_values<'a>(pending: &mut Vec<DisplayPart<'a>>, values: &'a [Value]) {
    for (index, value) in values.iter().enumerate().rev() {
        pending.push(DisplayPart::Value(value));
        if index > 0 {
            pending.push(DisplayPart::Text(", "));
        }
    }
}
