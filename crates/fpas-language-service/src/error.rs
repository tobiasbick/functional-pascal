//! Recoverable language-service failures.

use std::fmt;
use std::path::{Path, PathBuf};

/// A recoverable failure while loading or analyzing editor source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageServiceError {
    /// A caller cancelled bounded discovery or navigation work.
    Cancelled,
    /// A source file could not be read.
    SourceRead {
        /// Source path that could not be read.
        path: PathBuf,
        /// Underlying filesystem error text.
        message: String,
    },
    /// An editor update did not advance the open document version.
    StaleDocumentVersion {
        /// Document path receiving the stale update.
        path: PathBuf,
        /// Currently stored editor version.
        current: i64,
        /// Rejected editor version.
        received: i64,
    },
    /// A full-text update targeted a document that is not open.
    DocumentNotOpen {
        /// Path that has no open editor buffer.
        path: PathBuf,
    },
    /// Project-aware analysis could not be completed.
    Analysis {
        /// Document or manifest associated with the failure.
        path: PathBuf,
        /// Actionable project or semantic setup error.
        message: String,
    },
}

impl LanguageServiceError {
    pub(crate) fn source_read(path: &Path, error: impl fmt::Display) -> Self {
        Self::SourceRead {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    }

    pub(crate) fn analysis(path: &Path, message: impl Into<String>) -> Self {
        Self::Analysis {
            path: path.to_path_buf(),
            message: message.into(),
        }
    }
}

impl fmt::Display for LanguageServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("Language-service operation was cancelled."),
            Self::SourceRead { path, message } => {
                write!(
                    formatter,
                    "Cannot read source `{}`: {message}",
                    path.display()
                )
            }
            Self::StaleDocumentVersion {
                path,
                current,
                received,
            } => write!(
                formatter,
                "Stale document version {received} for `{}`; current version is {current}.",
                path.display()
            ),
            Self::DocumentNotOpen { path } => {
                write!(formatter, "Document `{}` is not open.", path.display())
            }
            Self::Analysis { path, message } => {
                write!(formatter, "Cannot analyze `{}`: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for LanguageServiceError {}
