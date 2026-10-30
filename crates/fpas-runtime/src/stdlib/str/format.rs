//! Numeric presentation and strict positional expansion for string.Format.
//!
//! **Documentation:** `docs/pascal/language/types/string/format-chars.md`

use crate::stdlib::error::{StdError, std_runtime_error};
use crate::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_FORMAT_MISMATCH;

mod number;
mod spec;

#[cfg(test)]
mod tests;

use spec::Spec;

/// Expand the template without changing the argument values or their order.
pub(super) fn apply_format(
    template: &str,
    args: &[Value],
    location: SourceLocation,
) -> Result<String, StdError> {
    let mut out = String::with_capacity(template.len());
    let mut arguments = args.iter();
    let mut characters = template.chars().peekable();
    let mut new_field_bytes = 0_usize;
    while let Some(character) = characters.next() {
        if character != '%' {
            out.push(character);
            continue;
        }
        if characters.peek() == Some(&'%') {
            characters.next();
            out.push('%');
            continue;
        }
        if characters.peek().is_none() {
            return Err(failure("trailing `%` at end of template string", location));
        }
        let spec = Spec::parse(&mut characters).map_err(|message| failure(message, location))?;
        let argument = arguments.next().ok_or_else(|| {
            failure(
                format!("not enough arguments for `%{}` specifier", spec.kind),
                location,
            )
        })?;
        if spec.kind == 's' {
            let Value::Str(text) = argument else {
                return Err(type_error('s', argument, location));
            };
            out.push_str(text);
            continue;
        }
        let text = numeric_argument(&spec, argument, location)?;
        if spec.extended() {
            new_field_bytes = new_field_bytes
                .checked_add(spec.width.max(text.len()))
                .filter(|total| *total <= spec::MAX_NEW_FIELD_BYTES)
                .ok_or_else(|| {
                    failure(
                        "new numeric fields exceed the maximum of 1000000 output bytes",
                        location,
                    )
                })?;
        }
        let padding = spec.width.saturating_sub(text.len());
        if spec.left {
            out.push_str(&text);
            out.extend(std::iter::repeat_n(' ', padding));
        } else if spec.zero {
            let digits = if let Some(digits) = text.strip_prefix('-') {
                out.push('-');
                digits
            } else {
                &text
            };
            out.extend(std::iter::repeat_n('0', padding));
            out.push_str(digits);
        } else {
            out.extend(std::iter::repeat_n(' ', padding));
            out.push_str(&text);
        }
    }
    if arguments.next().is_some() {
        return Err(failure("more arguments than format specifiers", location));
    }
    Ok(out)
}

fn numeric_argument(
    spec: &Spec,
    value: &Value,
    location: SourceLocation,
) -> Result<String, StdError> {
    match (spec.kind, value) {
        ('d', Value::Integer(value)) => Ok(value.to_string()),
        ('f', Value::Integer(value)) => Ok(spec.precision.map_or_else(
            || format_real(*value as f64),
            |precision| number::fixed_integer(*value, precision),
        )),
        ('e' | 'E', Value::Integer(value)) => Ok(number::scientific_integer(
            *value,
            spec.precision.unwrap_or(6),
            spec.kind == 'E',
        )),
        ('f' | 'e' | 'E', Value::Real(value)) => {
            if !value.is_finite() {
                return Err(failure("numeric arguments must be finite", location));
            }
            Ok(if spec.kind == 'f' {
                spec.precision.map_or_else(
                    || format_real(*value),
                    |precision| format!("{value:.precision$}"),
                )
            } else {
                number::scientific_real(*value, spec.precision.unwrap_or(6), spec.kind == 'E')
            })
        }
        _ => Err(type_error(spec.kind, value, location)),
    }
}

fn format_real(value: f64) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        format!("{value:.1}")
    } else {
        let text = format!("{value}");
        if text.contains('.') {
            text
        } else {
            format!("{text}.0")
        }
    }
}

fn type_error(kind: char, value: &Value, location: SourceLocation) -> StdError {
    let expected = match kind {
        'd' => "an integer",
        's' => "a string",
        _ => "a real or integer",
    };
    failure(
        format!(
            "`%{kind}` expects {expected}, got {}",
            value_type_name(value)
        ),
        location,
    )
}

fn failure(message: impl AsRef<str>, location: SourceLocation) -> StdError {
    std_runtime_error(
        RUNTIME_FORMAT_MISMATCH,
        format!("Format: {}", message.as_ref()),
        "Use numeric `%[-][0][width][.precision]d/f/e/E`, plain `%s`, or `%%`. Precision is for f/e/E; width <= 1000000 and precision <= 1024.",
        location,
    )
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "integer",
        Value::Real(_) => "real",
        Value::Boolean(_) => "boolean",
        Value::Str(_) => "string",
        Value::Bytes(_) => "Std.Bytes.ByteBuffer",
        Value::Array(_) => "array",
        Value::Dict(_) => "dict",
        Value::Record(_) => "record",
        Value::Enum(_) => "enum",
        Value::Unit => "unit",
        Value::ResultOk(_) | Value::ResultError(_) => "result",
        Value::OptionSome(_) | Value::OptionNone => "option",
        Value::Function(_) => "function",
        Value::Cell(_) => "cell",
        Value::Reference(_) => "reference",
        Value::Task(_) => "task",
        Value::OpaqueHandle(_) => "opaque handle",
    }
}
