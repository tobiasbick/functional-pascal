//! Recoverable document synchronization and query failures.

use std::fmt;
use std::path::PathBuf;

use fpas_language_service::{LanguageServiceError, RenameError};

use crate::convert::{FileUriError, PositionConversionError};

#[derive(Debug)]
pub(crate) enum DocumentRequestError {
    Service(LanguageServiceError),
    Rename(RenameError),
    Position(PositionConversionError),
    DocumentNotOpen { path: PathBuf },
    Task(String),
}

impl fmt::Display for DocumentRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Service(error) => error.fmt(formatter),
            Self::Rename(error) => error.fmt(formatter),
            Self::Position(error) => error.fmt(formatter),
            Self::DocumentNotOpen { path } => write!(
                formatter,
                "Cannot query `{}` because the document is not open.",
                path.display()
            ),
            Self::Task(message) => write!(formatter, "Language-service task failed: {message}"),
        }
    }
}

impl From<LanguageServiceError> for DocumentRequestError {
    fn from(error: LanguageServiceError) -> Self {
        Self::Service(error)
    }
}

impl From<RenameError> for DocumentRequestError {
    fn from(error: RenameError) -> Self {
        Self::Rename(error)
    }
}

impl From<PositionConversionError> for DocumentRequestError {
    fn from(error: PositionConversionError) -> Self {
        Self::Position(error)
    }
}

#[derive(Debug)]
pub(crate) enum DocumentSyncError {
    Uri(FileUriError),
    Service(LanguageServiceError),
    ExpectedOneFullChange { received: usize },
    IncrementalChange,
    DocumentNotOpen { path: PathBuf },
}

impl fmt::Display for DocumentSyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uri(error) => error.fmt(formatter),
            Self::Service(error) => error.fmt(formatter),
            Self::ExpectedOneFullChange { received } => write!(
                formatter,
                "Expected exactly one full-document content change, received {received}."
            ),
            Self::IncrementalChange => formatter.write_str(
                "Incremental text changes are unsupported; send one full-document change.",
            ),
            Self::DocumentNotOpen { path } => write!(
                formatter,
                "Cannot save `{}` because the document is not open.",
                path.display()
            ),
        }
    }
}

impl From<FileUriError> for DocumentSyncError {
    fn from(error: FileUriError) -> Self {
        Self::Uri(error)
    }
}

impl From<LanguageServiceError> for DocumentSyncError {
    fn from(error: LanguageServiceError) -> Self {
        Self::Service(error)
    }
}
