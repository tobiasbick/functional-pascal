//! Bounded, immutable Unicode patterns backed by Rust's regex engine.
mod matches;
mod replace;
#[cfg(test)]
mod tests;

use super::{
    error::{StdError, std_runtime_error},
    intrinsic_args::{IntrinsicCall, expect_str, pop_int, pop_value},
};
use crate::{RegexIntrinsic as Op, SourceLocation, Value};
use std::sync::Arc;

const MAX_TEXT: usize = 16 * 1024 * 1024;
const MAX_PATTERN: usize = 64 * 1024;
const MAX_COMPILED: usize = 8 * 1024 * 1024;
const MAX_ITEMS: usize = 100_000;
const MAX_GROUPS: usize = 256;

enum Failure {
    Vm(StdError),
    Pattern(String),
    Start(i64, usize),
    Limit(i64),
    Size(&'static str, usize),
}
impl From<StdError> for Failure {
    fn from(error: StdError) -> Self {
        Self::Vm(error)
    }
}

#[derive(Default)]
struct Budget {
    bytes: usize,
    items: usize,
}
impl Budget {
    fn text(&mut self, text: &str) -> Result<(), Failure> {
        self.bytes = self.bytes.saturating_add(text.len());
        check(self.bytes, MAX_TEXT, "OutputBytes")
    }
    fn item(&mut self) -> Result<(), Failure> {
        self.items += 1;
        check(self.items, MAX_ITEMS, "ResultItems")
    }
}
fn check(size: usize, limit: usize, name: &'static str) -> Result<(), Failure> {
    if size > limit {
        Err(Failure::Size(name, limit))
    } else {
        Ok(())
    }
}
fn mismatch(location: SourceLocation) -> StdError {
    std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        "Regex intrinsic arguments do not match its signature",
        "Pass a compiled Pattern and the documented argument types.",
        location,
    )
}
fn pattern(value: &Value, location: SourceLocation) -> Result<&regex::Regex, Failure> {
    match value {
        Value::Regex(value) => Ok(value),
        _ => Err(mismatch(location).into()),
    }
}
fn compile(expression: &str, value: &Value, location: SourceLocation) -> Result<Value, Failure> {
    check(expression.len(), MAX_PATTERN, "PatternBytes")?;
    let Value::Record(record) = value else {
        return Err(mismatch(location).into());
    };
    if record.body().layout.type_name != "Std.Text.Regex.Options" {
        return Err(mismatch(location).into());
    }
    let flags = record
        .body()
        .values
        .iter()
        .map(|value| match value {
            Value::Boolean(flag) => Ok(*flag),
            _ => Err(mismatch(location)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [case, multiline, dot, whitespace, greed, crlf] = flags.as_slice() else {
        return Err(mismatch(location).into());
    };
    let compiled = regex::RegexBuilder::new(expression)
        .case_insensitive(*case)
        .multi_line(*multiline)
        .dot_matches_new_line(*dot)
        .ignore_whitespace(*whitespace)
        .swap_greed(*greed)
        .crlf(*crlf)
        .size_limit(MAX_COMPILED)
        .dfa_size_limit(2 * 1024 * 1024)
        .nest_limit(250)
        .build()
        .map_err(|error| match error {
            regex::Error::CompiledTooBig(_) => Failure::Size("CompiledBytes", MAX_COMPILED),
            error => Failure::Pattern(error.to_string()),
        })?;
    check(compiled.captures_len(), MAX_GROUPS, "CaptureGroups")?;
    Ok(Value::Regex(Arc::new(compiled)))
}
fn start_byte(text: &str, start: i64) -> Result<usize, Failure> {
    let scalar = usize::try_from(start).ok();
    if let Some(scalar) = scalar {
        if let Some((byte, _)) = text.char_indices().nth(scalar) {
            return Ok(byte);
        }
        if scalar == text.chars().count() {
            return Ok(text.len());
        }
    }
    Err(Failure::Start(start, text.chars().count()))
}
fn limit(value: i64) -> Result<usize, Failure> {
    if value < 0 {
        Err(Failure::Limit(value))
    } else {
        Ok(usize::try_from(value).unwrap_or(usize::MAX))
    }
}

pub(super) fn run(
    operation: Op,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<(), StdError> {
    if operation == Op::DefaultOptions {
        call.push(call.factory().record(
            "Std.Text.Regex.Options",
            vec![Value::Boolean(false); 6],
            location,
        )?);
        return Ok(());
    }
    let result = execute(operation, call, location);
    let value = match result {
        Ok(value) => Value::result_ok(value),
        Err(Failure::Vm(error)) => return Err(error),
        Err(failure) => {
            let (variant, fields) = match failure {
                Failure::Pattern(detail) => ("InvalidPattern", vec![Value::Str(detail.into())]),
                Failure::Start(start, length) => (
                    "InvalidStart",
                    vec![Value::Integer(start), Value::Integer(length as i64)],
                ),
                Failure::Limit(limit) => ("InvalidLimit", vec![Value::Integer(limit)]),
                Failure::Size(name, limit) => (
                    "LimitExceeded",
                    vec![Value::Str(name.into()), Value::Integer(limit as i64)],
                ),
                Failure::Vm(_) => unreachable!("VM diagnostics returned above"),
            };
            Value::result_error(call.enumeration(
                "Std.Text.Regex.RegexError",
                variant,
                fields,
                location,
            )?)
        }
    };
    call.push(value);
    Ok(())
}

fn execute(
    operation: Op,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Value, Failure> {
    if operation == Op::Compile {
        let options = pop_value(call, location)?;
        let expression = expect_str(pop_value(call, location)?, location)?;
        return compile(expression, options, location);
    }
    if operation == Op::GroupNames {
        let pattern = pattern(pop_value(call, location)?, location)?;
        return Ok(Value::Array(
            pattern
                .capture_names()
                .map(|name| {
                    name.map_or(Value::OptionNone, |name| {
                        Value::option_some(Value::Str(name.into()))
                    })
                })
                .collect(),
        ));
    }
    if operation == Op::Escape {
        let text = expect_str(pop_value(call, location)?, location)?;
        check(text.len(), MAX_TEXT, "TextBytes")?;
        let escaped = regex::escape(text);
        check(escaped.len(), MAX_TEXT, "OutputBytes")?;
        return Ok(Value::Str(escaped.into()));
    }
    let count = match operation {
        Op::Find | Op::FindAll => pop_int(pop_value(call, location)?, location)?,
        Op::Split | Op::Replace => pop_int(pop_value(call, location)?, location)?,
        _ => 0,
    };
    let replacement = if operation == Op::Replace {
        Some(expect_str(pop_value(call, location)?, location)?)
    } else {
        None
    };
    let text = expect_str(pop_value(call, location)?, location)?;
    let pattern = pattern(pop_value(call, location)?, location)?;
    check(text.len(), MAX_TEXT, "TextBytes")?;
    match operation {
        Op::IsMatch => Ok(Value::Boolean(pattern.is_match(text))),
        Op::Find => {
            let start = start_byte(text, count)?;
            let mut budget = Budget::default();
            match pattern.captures_at(text, start) {
                None => Ok(Value::OptionNone),
                Some(captures) => Ok(Value::option_some(matches::value(
                    pattern,
                    text,
                    &captures,
                    &mut matches::Cursor::default(),
                    &mut budget,
                    call,
                    location,
                )?)),
            }
        }
        Op::FindAll => matches::all(pattern, text, start_byte(text, count)?, call, location),
        Op::Replace => replace::replace(
            pattern,
            text,
            replacement.map_or("", |s| s.as_ref()),
            limit(count)?,
        ),
        Op::Split => replace::split(pattern, text, limit(count)?),
        _ => unreachable!("other operations returned before search dispatch"),
    }
}
