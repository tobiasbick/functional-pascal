//! One bounded source line and its cell-aligned diagnostic marker.

use super::{TextStyle, display};
use crate::{Diagnostic, SourcePosition, SourceRange};

/// Appends one valid source excerpt without changing the existing diagnostic on failure.
pub(super) fn push(output: &mut String, source: &str, diagnostic: &Diagnostic, style: TextStyle) {
    let Some(span) = diagnostic.span else {
        return;
    };
    let Some(range) = SourceRange::resolve(span, Some(source)) else {
        return;
    };
    // Reject stale/inconsistent producer coordinates instead of showing a different line.
    if range.start.line != span.line() || range.start.column != span.column() {
        return;
    }
    let Some(offset) = (SourcePosition {
        line: range.start.line,
        column: 1,
    })
    .offset_in(source) else {
        return;
    };
    let line = source[offset..].split(['\r', '\n']).next().unwrap_or("");
    let line = display::line(line);
    let Some(start) = line.offset(range.start.column) else {
        return;
    };
    let continued = range.end.is_some_and(|end| end.line > range.start.line);
    let end = match range.end {
        Some(end) if !continued => {
            let Some(end) = line.offset(end.column) else {
                return;
            };
            end
        }
        Some(_) => line.end(),
        None => start,
    };
    let window = display::window(&line, start, end);
    let number = range.start.line.to_string();
    output.push_str(&format!(
        "\n  {number} | {}\n  {} | {}",
        window.text,
        " ".repeat(number.len()),
        " ".repeat(window.caret)
    ));
    style.push(output, &"^".repeat(window.length), diagnostic.is_warning());
    if continued {
        output.push_str(" ...");
    }
}
