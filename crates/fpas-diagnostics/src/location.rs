//! Source locations and spans shared across diagnostics.

fn validate_one_based_location(line: u32, column: u32) {
    assert!(line > 0, "source line must be 1-based");
    assert!(column > 0, "source column must be 1-based");
}

/// A 1-based location within a source input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub source_id: u32,
}

impl SourceLocation {
    /// Creates a location in the primary source input.
    #[must_use]
    pub fn new(line: u32, column: u32) -> Self {
        Self::new_with_source(line, column, 0)
    }

    /// Creates a location with an explicit source identifier.
    #[must_use]
    pub fn new_with_source(line: u32, column: u32, source_id: u32) -> Self {
        validate_one_based_location(line, column);
        Self {
            line,
            column,
            source_id,
        }
    }
}

impl From<(u32, u32)> for SourceLocation {
    fn from((line, column): (u32, u32)) -> Self {
        Self::new(line, column)
    }
}

/// A source span with byte offsets and a 1-based start location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub offset: usize,
    pub length: usize,
    pub line: u32,
    pub column: u32,
    pub source_id: u32,
}

impl SourceSpan {
    /// Creates a span in the primary source input.
    #[must_use]
    pub fn new(offset: usize, length: usize, line: u32, column: u32) -> Self {
        Self::new_with_source(offset, length, line, column, 0)
    }

    /// Creates a span with an explicit source identifier.
    #[must_use]
    pub fn new_with_source(
        offset: usize,
        length: usize,
        line: u32,
        column: u32,
        source_id: u32,
    ) -> Self {
        validate_one_based_location(line, column);
        Self {
            offset,
            length,
            line,
            column,
            source_id,
        }
    }

    /// Placeholder span when only a [`SourceLocation`] is known (byte offsets unset).
    #[must_use]
    pub fn synthetic_from_location(location: SourceLocation) -> Self {
        Self::new_with_source(0, 1, location.line, location.column, location.source_id)
    }

    /// Returns the starting location of this span.
    #[must_use]
    pub fn location(self) -> SourceLocation {
        SourceLocation::new_with_source(self.line, self.column, self.source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceLocation, SourceSpan};

    #[test]
    fn source_location_from_tuple() {
        let location = SourceLocation::from((12, 34));
        assert_eq!(location, SourceLocation::new(12, 34));
    }

    #[test]
    fn source_span_location_returns_line_and_column() {
        let span = SourceSpan::new(7, 5, 21, 3);
        assert_eq!(span.location(), SourceLocation::new(21, 3));
    }

    #[test]
    fn source_span_location_preserves_source_id() {
        let span = SourceSpan::new_with_source(7, 5, 21, 3, 9);
        assert_eq!(span.location(), SourceLocation::new_with_source(21, 3, 9));
    }

    #[test]
    fn synthetic_from_location_uses_placeholder_offsets() {
        let location = SourceLocation::new_with_source(4, 2, 3);
        let span = SourceSpan::synthetic_from_location(location);
        assert_eq!(span.offset, 0);
        assert_eq!(span.length, 1);
        assert_eq!(span.location(), location);
    }

    #[test]
    #[should_panic(expected = "source line must be 1-based")]
    fn source_location_rejects_zero_line() {
        let _ = SourceLocation::new(0, 1);
    }

    #[test]
    #[should_panic(expected = "source column must be 1-based")]
    fn source_location_rejects_zero_column() {
        let _ = SourceLocation::new(1, 0);
    }
}
