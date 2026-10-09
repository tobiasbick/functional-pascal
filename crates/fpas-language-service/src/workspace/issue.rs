//! Recoverable discovery issues retain authoritative project diagnostics.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use std::path::{Path, PathBuf};

use fpas_project::ProjectError;

use crate::LanguageServiceError;
use crate::document::normalized_path;

/// A recoverable context-loading problem that does not terminate the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIssue {
    /// Manifest or discovery path associated with the problem.
    pub path: PathBuf,
    /// Actionable text for callers displaying a discovery issue.
    pub message: String,
    error: Option<ProjectError>,
}

impl WorkspaceIssue {
    /// Creates a positionless issue produced by editor discovery.
    pub(super) fn new(path: &Path, message: impl Into<String>) -> Self {
        Self {
            path: normalized_path(path),
            message: message.into(),
            error: None,
        }
    }

    /// Retains a project failure alongside its display text and discovery owner.
    pub(super) fn from_project(path: &Path, error: ProjectError) -> Self {
        Self {
            path: normalized_path(path),
            message: error.to_string(),
            error: Some(error),
        }
    }

    /// Returns the original records when discovery failed in the project loader.
    pub(crate) fn into_analysis_error(self) -> LanguageServiceError {
        self.error.map_or_else(
            || LanguageServiceError::analysis(&self.path, self.message),
            LanguageServiceError::from,
        )
    }
}
