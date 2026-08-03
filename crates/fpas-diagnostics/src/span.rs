//! Validated byte spans and their one-based source locations.

use core::fmt;

use crate::location::{
    SourceLocation, SourceLocationError, assert_one_based_location, validate_one_based_location,
};

/// Error returned when a source span has an invalid location or overflowing byte range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceSpanError {
    /// The source line or column is not one-based.
    Location(SourceLocationError),
    /// The exclusive end offset cannot be represented as a [`usize`].
    EndOverflow {
        /// Zero-based byte offset supplied for the span.
        offset: usize,
        /// Byte length supplied for the span.
        length: usize,
    },
}

impl fmt::Display for SourceSpanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Location(error) => error.fmt(formatter),
            Self::EndOverflow { offset, length } => write!(
                formatter,
                "source span end overflows usize: offset {offset}, length {length}"
            ),
        }
    }
}

impl std::error::Error for SourceSpanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Location(error) => Some(error),
            Self::EndOverflow { .. } => None,
        }
    }
}

impl From<SourceLocationError> for SourceSpanError {
    fn from(error: SourceLocationError) -> Self {
        Self::Location(error)
    }
}

/// A validated source span with byte offsets and a 1-based start location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    offset: usize,
    length: usize,
    line: u32,
    column: u32,
    source_id: u32,
}

impl SourceSpan {
    /// Creates a span in the primary source input.
    ///
    /// Prefer [`Self::try_new`] for dynamic or untrusted values.
    ///
    /// # Panics
    ///
    /// Panics when `line` or `column` is zero, or when `offset + length` overflows [`usize`].
    #[must_use]
    pub fn new(offset: usize, length: usize, line: u32, column: u32) -> Self {
        Self::new_with_source(offset, length, line, column, 0)
    }

    /// Creates a span with an explicit source identifier.
    ///
    /// Prefer [`Self::try_new_with_source`] for dynamic or untrusted values.
    ///
    /// # Panics
    ///
    /// Panics when `line` or `column` is zero, or when `offset + length` overflows [`usize`].
    #[must_use]
    pub fn new_with_source(
        offset: usize,
        length: usize,
        line: u32,
        column: u32,
        source_id: u32,
    ) -> Self {
        assert_one_based_location(line, column);
        assert!(
            offset.checked_add(length).is_some(),
            "source span end overflows usize"
        );
        Self {
            offset,
            length,
            line,
            column,
            source_id,
        }
    }

    /// Tries to create a span in the primary source input.
    ///
    /// # Errors
    ///
    /// Returns [`SourceSpanError`] for zero coordinates or an overflowing exclusive end offset.
    pub fn try_new(
        offset: usize,
        length: usize,
        line: u32,
        column: u32,
    ) -> Result<Self, SourceSpanError> {
        Self::try_new_with_source(offset, length, line, column, 0)
    }

    /// Tries to create a span with an explicit source identifier.
    ///
    /// # Errors
    ///
    /// Returns [`SourceSpanError`] for zero coordinates or an overflowing exclusive end offset.
    pub fn try_new_with_source(
        offset: usize,
        length: usize,
        line: u32,
        column: u32,
        source_id: u32,
    ) -> Result<Self, SourceSpanError> {
        validate_one_based_location(line, column)?;
        offset
            .checked_add(length)
            .ok_or(SourceSpanError::EndOverflow { offset, length })?;
        Ok(Self {
            offset,
            length,
            line,
            column,
            source_id,
        })
    }

    /// Creates a placeholder span when only a [`SourceLocation`] is known.
    #[must_use]
    pub fn synthetic_from_location(location: SourceLocation) -> Self {
        Self::new_with_source(
            0,
            1,
            location.line(),
            location.column(),
            location.source_id(),
        )
    }

    /// Returns the zero-based byte offset.
    #[must_use]
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Returns the byte length.
    #[must_use]
    pub const fn length(self) -> usize {
        self.length
    }

    /// Returns the exclusive byte end offset.
    ///
    /// Construction validates that this addition cannot overflow.
    #[must_use]
    pub const fn end(self) -> usize {
        self.offset + self.length
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }

    /// Returns the one-based source column.
    #[must_use]
    pub const fn column(self) -> u32 {
        self.column
    }

    /// Returns the identifier of the containing source input.
    #[must_use]
    pub const fn source_id(self) -> u32 {
        self.source_id
    }

    /// Returns this span associated with `source_id`.
    #[must_use]
    pub const fn with_source_id(self, source_id: u32) -> Self {
        Self { source_id, ..self }
    }

    /// Returns the starting location of this span.
    #[must_use]
    pub fn location(self) -> SourceLocation {
        SourceLocation::new_with_source(self.line, self.column, self.source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceSpan, SourceSpanError};
    use crate::SourceLocation;

    #[test]
    fn source_span_location_preserves_coordinates_and_source_id() {
        let span = SourceSpan::new_with_source(7, 5, 21, 3, 9);
        assert_eq!(span.location(), SourceLocation::new_with_source(21, 3, 9));
    }

    #[test]
    fn synthetic_from_location_uses_placeholder_offsets() {
        let location = SourceLocation::new_with_source(4, 2, 3);
        let span = SourceSpan::synthetic_from_location(location);
        assert_eq!(span.offset(), 0);
        assert_eq!(span.length(), 1);
        assert_eq!(span.end(), 1);
        assert_eq!(span.location(), location);
    }

    #[test]
    fn source_span_accepts_representable_end_boundaries() {
        assert_eq!(SourceSpan::new(usize::MAX, 0, 1, 1).end(), usize::MAX);
        assert_eq!(SourceSpan::new(usize::MAX - 1, 1, 1, 1).end(), usize::MAX);
    }

    #[test]
    fn source_span_try_new_rejects_overflow() {
        assert_eq!(
            SourceSpan::try_new(usize::MAX, 1, 1, 1),
            Err(SourceSpanError::EndOverflow {
                offset: usize::MAX,
                length: 1,
            })
        );
    }

    #[test]
    #[should_panic(expected = "source span end overflows usize")]
    fn source_span_new_rejects_overflow() {
        let _ = SourceSpan::new(usize::MAX, 1, 1, 1);
    }
}
