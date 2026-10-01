//! Project failures preserve source diagnostics until the caller chooses a renderer.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::Diagnostic;

/// A project-loading or unit-graph failure.
///
/// Source reading, lexing and parsing retain shared diagnostics. Other project
/// validation failures currently retain their original text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectError {
    failure: Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Failure {
    Source {
        path: PathBuf,
        diagnostics: Vec<Diagnostic>,
    },
    Message(String),
}

impl ProjectError {
    /// Associates diagnostics produced from one source with its authoritative path.
    pub(crate) fn from_source(path: &Path, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            failure: Failure::Source {
                path: path.to_path_buf(),
                diagnostics,
            },
        }
    }

    /// Returns source diagnostics in producer order, or an empty slice for text failures.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match &self.failure {
            Failure::Source { diagnostics, .. } => diagnostics,
            Failure::Message(_) => &[],
        }
    }

    /// Returns the source file that produced the diagnostics, even without a position.
    #[must_use]
    pub fn source_path(&self) -> Option<&Path> {
        match &self.failure {
            Failure::Source { path, .. } => Some(path),
            Failure::Message(_) => None,
        }
    }
}

impl From<String> for ProjectError {
    fn from(message: String) -> Self {
        Self {
            failure: Failure::Message(message),
        }
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.failure {
            Failure::Message(message) => formatter.write_str(message),
            Failure::Source { path, diagnostics } => {
                for (index, diagnostic) in diagnostics.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str("\n")?;
                    }
                    formatter.write_str(&fpas_diagnostics::render(
                        &path.to_string_lossy(),
                        diagnostic,
                    ))?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ProjectError {}
