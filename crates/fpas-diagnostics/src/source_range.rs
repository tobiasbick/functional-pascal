//! Unicode-scalar source coordinates for diagnostic output.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use serde::Serialize;

use crate::SourceSpan;

/// One-based line and Unicode scalar column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SourcePosition {
    /// One-based source line.
    pub line: u32,
    /// One-based Unicode scalar column (not bytes, UTF-16 units, or display cells).
    pub column: u32,
}

impl SourcePosition {
    /// Resolves this scalar position to a UTF-8 byte offset in the supplied text.
    /// Returns `None` for positions outside the source.
    #[must_use]
    pub fn offset_in(self, source: &str) -> Option<usize> {
        let mut position = Self { line: 1, column: 1 };
        let mut previous_cr = false;
        for (offset, character) in source.char_indices() {
            // The LF half of CRLF has no separate source position.
            if position == self && !(previous_cr && character == '\n') {
                return Some(offset);
            }
            advance(&mut position, character, previous_cr)?;
            previous_cr = character == '\r';
        }
        (position == self).then_some(source.len())
    }
}

/// Source identity and available coordinates; the end is exclusive when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SourceRange {
    /// Identity within the compilation's source table.
    pub source_id: u32,
    /// Start of the diagnostic.
    pub start: SourcePosition,
    /// Exclusive end, absent for runtime point locations without byte ranges.
    pub end: Option<SourcePosition>,
}

impl SourceRange {
    /// Resolves a byte span using the matching source text.
    ///
    /// Returns `None` for an invalid UTF-8 boundary or out-of-bounds range.
    /// Synthetic spans preserve their known point without fabricating an end.
    /// Without text, a real span retains only its producer's start location.
    #[must_use]
    pub fn resolve(span: SourceSpan, source: Option<&str>) -> Option<Self> {
        let mut range = Self {
            source_id: span.source_id(),
            start: SourcePosition {
                line: span.line(),
                column: span.column(),
            },
            end: None,
        };
        if !span.is_synthetic()
            && let Some(source) = source
        {
            range.start = position_at(source, span.offset())?;
            range.end = Some(position_at(source, span.end())?);
        }
        Some(range)
    }
}

fn position_at(source: &str, offset: usize) -> Option<SourcePosition> {
    let prefix = source.get(..offset)?;
    let mut position = SourcePosition { line: 1, column: 1 };
    let mut previous_cr = false;
    for character in prefix.chars() {
        advance(&mut position, character, previous_cr)?;
        previous_cr = character == '\r';
    }
    Some(position)
}

fn advance(position: &mut SourcePosition, character: char, previous_cr: bool) -> Option<()> {
    match character {
        '\n' if previous_cr => {}
        '\r' | '\n' => {
            position.line = position.line.checked_add(1)?;
            position.column = 1;
        }
        _ => position.column = position.column.checked_add(1)?,
    }
    Some(())
}
