//! Validated one-based source locations shared across diagnostics.

use core::fmt;

/// Error returned when a source location is not one-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLocationError {
    /// The source line was zero.
    ZeroLine,
    /// The source column was zero.
    ZeroColumn,
}

impl fmt::Display for SourceLocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroLine => "source line must be 1-based",
            Self::ZeroColumn => "source column must be 1-based",
        })
    }
}

impl std::error::Error for SourceLocationError {}

pub(super) fn validate_one_based_location(
    line: u32,
    column: u32,
) -> Result<(), SourceLocationError> {
    if line == 0 {
        return Err(SourceLocationError::ZeroLine);
    }
    if column == 0 {
        return Err(SourceLocationError::ZeroColumn);
    }
    Ok(())
}

pub(super) fn assert_one_based_location(line: u32, column: u32) {
    assert!(line > 0, "source line must be 1-based");
    assert!(column > 0, "source column must be 1-based");
}

/// A validated 1-based location within a source input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    line: u32,
    column: u32,
    source_id: u32,
}

impl SourceLocation {
    /// Creates a location in the primary source input.
    ///
    /// Prefer [`Self::try_new`] for dynamic or untrusted coordinates.
    ///
    /// # Panics
    ///
    /// Panics when `line` or `column` is zero.
    #[must_use]
    pub fn new(line: u32, column: u32) -> Self {
        Self::new_with_source(line, column, 0)
    }

    /// Creates a location with an explicit source identifier.
    ///
    /// Prefer [`Self::try_new_with_source`] for dynamic or untrusted coordinates.
    ///
    /// # Panics
    ///
    /// Panics when `line` or `column` is zero.
    #[must_use]
    pub fn new_with_source(line: u32, column: u32, source_id: u32) -> Self {
        assert_one_based_location(line, column);
        Self {
            line,
            column,
            source_id,
        }
    }

    /// Tries to create a location in the primary source input.
    ///
    /// # Errors
    ///
    /// Returns [`SourceLocationError`] when `line` or `column` is zero.
    pub fn try_new(line: u32, column: u32) -> Result<Self, SourceLocationError> {
        Self::try_new_with_source(line, column, 0)
    }

    /// Tries to create a location with an explicit source identifier.
    ///
    /// # Errors
    ///
    /// Returns [`SourceLocationError`] when `line` or `column` is zero.
    pub fn try_new_with_source(
        line: u32,
        column: u32,
        source_id: u32,
    ) -> Result<Self, SourceLocationError> {
        validate_one_based_location(line, column)?;
        Ok(Self {
            line,
            column,
            source_id,
        })
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

    /// Returns this location associated with `source_id`.
    #[must_use]
    pub const fn with_source_id(self, source_id: u32) -> Self {
        Self { source_id, ..self }
    }
}

impl TryFrom<(u32, u32)> for SourceLocation {
    type Error = SourceLocationError;

    fn try_from((line, column): (u32, u32)) -> Result<Self, Self::Error> {
        Self::try_new(line, column)
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceLocation, SourceLocationError};

    #[test]
    fn source_location_try_from_tuple_validates_coordinates() {
        assert_eq!(
            SourceLocation::try_from((12, 34)),
            Ok(SourceLocation::new(12, 34))
        );
        assert_eq!(
            SourceLocation::try_from((0, 34)),
            Err(SourceLocationError::ZeroLine)
        );
    }

    #[test]
    #[should_panic(expected = "source line must be 1-based")]
    fn source_location_new_rejects_zero_line() {
        let _ = SourceLocation::new(0, 1);
    }

    #[test]
    #[should_panic(expected = "source column must be 1-based")]
    fn source_location_new_rejects_zero_column() {
        let _ = SourceLocation::new(1, 0);
    }
}
