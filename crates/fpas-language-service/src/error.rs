//! Recoverable language-service failures.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::{
    INTERNAL_PROJECT_INVARIANT_FAILURE, PROJECT_SOURCE_INVALID_UTF8, PROJECT_SOURCE_READ_FAILED,
};
use fpas_diagnostics::{Diagnostic, FileDiagnostic};
use fpas_project::ProjectError;

/// A recoverable failure while loading or analyzing editor source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageServiceError {
    /// Original project-loading or graph failure, including its source attribution.
    Project(ProjectError),
    /// A caller cancelled bounded discovery or navigation work.
    Cancelled,
    /// A source file could not be read.
    SourceRead {
        /// Source path that could not be read.
        path: PathBuf,
        /// Coded read failure with its original message and correction hint.
        diagnostic: Box<Diagnostic>,
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
    /// Returns original project records or a coded, positionless service failure.
    ///
    /// **Documentation:** `docs/pascal/tools/diagnostics.md`
    #[must_use]
    pub fn diagnostics(&self) -> Vec<FileDiagnostic> {
        let (code, message, path) = match self {
            Self::Project(error) => {
                return error
                    .diagnostics()
                    .iter()
                    .cloned()
                    .map(|diagnostic| {
                        FileDiagnostic::new(diagnostic, error.source_path().map(Path::to_path_buf))
                    })
                    .collect();
            }
            Self::Cancelled => return Vec::new(),
            Self::SourceRead { path, diagnostic } => {
                return vec![FileDiagnostic::new(
                    diagnostic.as_ref().clone(),
                    Some(path.clone()),
                )];
            }
            Self::Analysis { path, message } => (
                INTERNAL_PROJECT_INVARIANT_FAILURE,
                message.clone(),
                Some(path.clone()),
            ),
            Self::StaleDocumentVersion { path, .. } | Self::DocumentNotOpen { path } => (
                INTERNAL_PROJECT_INVARIANT_FAILURE,
                self.to_string(),
                Some(path.clone()),
            ),
        };
        vec![FileDiagnostic::new(
            Diagnostic::error_without_source(code, message, None),
            path,
        )]
    }

    /// Classifies filesystem and encoding failures without discarding their hints.
    pub(crate) fn source_read(path: &Path, error: std::io::Error) -> Self {
        let (code, hint) = if error.kind() == std::io::ErrorKind::InvalidData {
            (
                PROJECT_SOURCE_INVALID_UTF8,
                "Save the source file as UTF-8.",
            )
        } else {
            (
                PROJECT_SOURCE_READ_FAILED,
                "Check that the source file exists and is readable.",
            )
        };
        Self::SourceRead {
            path: path.to_path_buf(),
            diagnostic: Box::new(Diagnostic::error_without_source(
                code,
                format!("Cannot read source: {error}"),
                Some(hint.to_string()),
            )),
        }
    }

    pub(crate) fn analysis(path: &Path, message: impl Into<String>) -> Self {
        Self::Analysis {
            path: path.to_path_buf(),
            message: message.into(),
        }
    }
}

impl From<ProjectError> for LanguageServiceError {
    fn from(error: ProjectError) -> Self {
        Self::Project(error)
    }
}

impl fmt::Display for LanguageServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Project(error) => error.fmt(formatter),
            Self::Cancelled => formatter.write_str("Language-service operation was cancelled."),
            Self::SourceRead { path, diagnostic } => {
                write!(formatter, "{}: {}", path.display(), diagnostic.message)
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
