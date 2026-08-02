//! Reusable UTF-8 byte-offset line index.

use std::ops::Range;
use std::sync::Arc;

/// A zero-based source position whose column is a UTF-8 byte offset.
///
/// LSP UTF-16 conversion deliberately belongs to the transport crate. This position remains tied
/// to compiler byte spans and never depends on an editor protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextPosition {
    /// Zero-based line number.
    pub line: usize,
    /// Zero-based UTF-8 byte column within the line.
    pub byte_column: usize,
}

/// A half-open source range expressed as UTF-8 byte positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
    /// Inclusive range start.
    pub start: TextPosition,
    /// Exclusive range end.
    pub end: TextPosition,
}

/// Source-bound line-start table for translating compiler byte offsets without rescanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    source: Arc<str>,
}

impl LineIndex {
    /// Builds an index that owns the exact UTF-8 source text used by later translations.
    #[must_use]
    pub fn new(source: impl Into<Arc<str>>) -> Self {
        let source = source.into();
        let mut line_starts = vec![0];
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        );
        Self {
            line_starts,
            source,
        }
    }

    /// Returns the number of logical lines, including a trailing empty line after `\n`.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns the byte offset where a zero-based line begins.
    #[must_use]
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Returns the content byte range for a line, excluding `\n` and an optional preceding `\r`.
    #[must_use]
    pub fn line_range(&self, line: usize) -> Option<Range<usize>> {
        let start = self.line_start(line)?;
        let mut end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.source.len());
        if end > start && self.source.as_bytes().get(end - 1) == Some(&b'\n') {
            end -= 1;
        }
        if end > start && self.source.as_bytes().get(end - 1) == Some(&b'\r') {
            end -= 1;
        }
        Some(start..end)
    }

    /// Translates a UTF-8 byte offset to a zero-based line and byte column.
    #[must_use]
    pub fn position(&self, offset: usize) -> Option<TextPosition> {
        if offset > self.source.len() || !self.source.is_char_boundary(offset) {
            return None;
        }
        let line = self.line_starts.partition_point(|start| *start <= offset) - 1;
        Some(TextPosition {
            line,
            byte_column: offset - self.line_starts[line],
        })
    }

    /// Translates a zero-based line and UTF-8 byte column to a compiler byte offset.
    #[must_use]
    pub fn offset(&self, position: TextPosition) -> Option<usize> {
        let line = self.line_range(position.line)?;
        let offset = line.start.checked_add(position.byte_column)?;
        (offset <= line.end && self.source.is_char_boundary(offset)).then_some(offset)
    }

    /// Converts a half-open compiler byte span to an editor-independent text range.
    #[must_use]
    pub fn text_range(&self, offset: usize, length: usize) -> Option<TextRange> {
        let end = offset.checked_add(length)?;
        Some(TextRange {
            start: self.position(offset)?,
            end: self.position(end)?,
        })
    }
}
