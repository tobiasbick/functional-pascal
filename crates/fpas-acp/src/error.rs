//! Bounded codec failures whose ordinary text never includes protocol contents.

use std::fmt;

/// Category of a rejected value at the ACP codec boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Invalid JSON syntax or a value rejected by the pinned schema.
    InvalidMessage,
    /// A JSON object repeats a decoded key.
    DuplicateKey,
    /// A value exceeds the maximum nesting depth.
    DepthExceeded,
    /// A value exceeds the configured byte budget.
    LimitExceeded,
    /// Invalid or out-of-range decimal integer.
    InvalidInteger,
    /// A known binary64 field is not finite.
    NonFinite,
}

/// Operational codec error with safe structural position information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodecError {
    /// Error category, independent of diagnostic wording.
    pub kind: ErrorKind,
    /// One-based JSON line, when the parser supplies it.
    pub line: Option<usize>,
    /// One-based JSON column, when the parser supplies it.
    pub column: Option<usize>,
}

impl CodecError {
    /// Construct a failure without retaining an input body or secret field value.
    #[must_use]
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            line: None,
            column: None,
        }
    }

    pub(crate) fn json(error: serde_json::Error) -> Self {
        Self {
            kind: ErrorKind::InvalidMessage,
            line: (error.line() != 0).then_some(error.line()),
            column: (error.column() != 0).then_some(error.column()),
        }
    }
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ACP {:?}", self.kind)?;
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(formatter, " at {line}:{column}")?;
        }
        Ok(())
    }
}

impl std::error::Error for CodecError {}

impl serde::ser::Error for CodecError {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self::new(ErrorKind::InvalidMessage)
    }
}
