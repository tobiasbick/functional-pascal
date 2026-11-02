//! Match construction converts sorted byte boundaries to Unicode scalar positions once.
use super::{Budget, Failure};
use crate::{SourceLocation, Value, stdlib::intrinsic_args::IntrinsicCall};

#[derive(Default)]
pub(super) struct Cursor {
    byte: usize,
    scalar: usize,
}

pub(super) fn value(
    pattern: &regex::Regex,
    text: &str,
    captures: &regex::Captures<'_>,
    cursor: &mut Cursor,
    budget: &mut Budget,
    call: &IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Value, Failure> {
    budget.item()?;
    let mut boundaries = captures
        .iter()
        .flatten()
        .flat_map(|span| [span.start(), span.end()])
        .collect::<Vec<_>>();
    boundaries.sort_unstable();
    boundaries.dedup();
    let offsets = boundaries
        .into_iter()
        .map(|byte| {
            cursor.scalar += text[cursor.byte..byte].chars().count();
            cursor.byte = byte;
            (byte, cursor.scalar)
        })
        .collect::<Vec<_>>();
    let position = |byte| {
        let index = offsets
            .binary_search_by_key(&byte, |(offset, _)| *offset)
            .unwrap_or_else(|_| unreachable!("capture boundary was indexed"));
        offsets[index].1
    };
    let span_value = |span: regex::Match<'_>, budget: &mut Budget| -> Result<Value, Failure> {
        budget.text(span.as_str())?;
        Ok(call.factory().record(
            "Std.Text.Regex.Span",
            vec![
                Value::Integer(position(span.start()) as i64),
                Value::Integer((position(span.end()) - position(span.start())) as i64),
                Value::Str(span.as_str().into()),
            ],
            location,
        )?)
    };
    let whole = captures
        .get(0)
        .unwrap_or_else(|| unreachable!("group zero always exists"));
    let whole = span_value(whole, budget)?;
    let mut groups = Vec::new();
    for (name, span) in pattern.capture_names().zip(captures.iter()) {
        budget.item()?;
        let span = match span {
            Some(span) => Value::option_some(span_value(span, budget)?),
            None => Value::OptionNone,
        };
        let name = if let Some(name) = name {
            budget.text(name)?;
            Value::option_some(Value::Str(name.into()))
        } else {
            Value::OptionNone
        };
        groups.push(
            call.factory()
                .record("Std.Text.Regex.Capture", vec![name, span], location)?,
        );
    }
    Ok(call.factory().record(
        "Std.Text.Regex.Match",
        vec![whole, Value::Array(groups.into())],
        location,
    )?)
}

pub(super) fn all(
    pattern: &regex::Regex,
    text: &str,
    start: usize,
    call: &IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Value, Failure> {
    let mut found = Vec::new();
    let mut budget = Budget::default();
    let mut cursor = Cursor::default();
    let mut next = start;
    let mut previous_end = None;
    while next <= text.len() {
        let Some(captures) = pattern.captures_at(text, next) else {
            break;
        };
        let whole = captures
            .get(0)
            .unwrap_or_else(|| unreachable!("group zero always exists"));
        let empty = whole.is_empty();
        if empty && previous_end == Some(whole.end()) {
            next = advance(text, whole.end());
            continue;
        }
        found.push(value(
            pattern,
            text,
            &captures,
            &mut cursor,
            &mut budget,
            call,
            location,
        )?);
        previous_end = Some(whole.end());
        next = if empty {
            advance(text, whole.end())
        } else {
            whole.end()
        };
    }
    Ok(Value::Array(found.into()))
}
fn advance(text: &str, byte: usize) -> usize {
    byte + text[byte..].chars().next().map_or(1, char::len_utf8)
}
