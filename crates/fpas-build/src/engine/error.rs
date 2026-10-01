//! Build failures retain diagnostic records until an output boundary renders them.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::Diagnostic;
use fpas_linker::LinkError;

/// A compiler or parser diagnostic and its known source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildDiagnostic {
    /// Original diagnostic, including its code, source ID and structured details.
    pub diagnostic: Diagnostic,
    /// Source file when supplied by the producing build operation.
    pub path: Option<PathBuf>,
}

/// Incremental build failure with preserved compiler, parser or linker details.
///
/// Filesystem and project-loading failures may still contain only text. An empty
/// [`Self::diagnostics`] slice does not imply that the build succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    failure: Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Failure {
    Message(String),
    Diagnostics {
        records: Vec<BuildDiagnostic>,
        display_paths: bool,
    },
    Link(Box<LinkError>),
}

impl BuildError {
    /// Retains a failure from a producer that currently reports only text.
    pub(crate) fn new(detail: impl Into<String>) -> Self {
        Self {
            failure: Failure::Message(detail.into()),
        }
    }

    /// Retains producer records and associates only matching source IDs with a path.
    pub(crate) fn from_diagnostics(
        diagnostics: Vec<Diagnostic>,
        source: Option<(u32, &Path)>,
    ) -> Self {
        Self {
            failure: Failure::Diagnostics {
                records: diagnostics
                    .into_iter()
                    .map(|diagnostic| {
                        let path = source
                            .filter(|(id, _)| {
                                diagnostic.span.is_some_and(|span| span.source_id() == *id)
                            })
                            .map(|(_, path)| path.to_path_buf());
                        BuildDiagnostic { diagnostic, path }
                    })
                    .collect(),
                display_paths: true,
            },
        }
    }

    /// Retains root compiler records while leaving text file context to the caller.
    pub(crate) fn from_program_diagnostics(
        diagnostics: Vec<Diagnostic>,
        path: Option<&Path>,
    ) -> Self {
        let mut error = Self::from_diagnostics(diagnostics, path.map(|path| (0, path)));
        if let Failure::Diagnostics { display_paths, .. } = &mut error.failure {
            // Program callers already supply the main-file context in text output.
            *display_paths = false;
        }
        error
    }

    /// Returns the original compiler/parser records in producer order.
    ///
    /// Non-diagnostic failures return an empty slice. No records are recovered
    /// by parsing formatted text.
    #[must_use]
    pub fn diagnostics(&self) -> &[BuildDiagnostic] {
        match &self.failure {
            Failure::Diagnostics { records, .. } => records,
            _ => &[],
        }
    }

    /// Returns the original typed linker failure, when linking failed.
    #[must_use]
    pub fn link_error(&self) -> Option<&LinkError> {
        match &self.failure {
            Failure::Link(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LinkError> for BuildError {
    fn from(error: LinkError) -> Self {
        Self {
            failure: Failure::Link(Box::new(error)),
        }
    }
}

impl From<fpas_project::ProjectError> for BuildError {
    fn from(error: fpas_project::ProjectError) -> Self {
        if let Some(path) = error.source_path() {
            Self {
                failure: Failure::Diagnostics {
                    records: error
                        .diagnostics()
                        .iter()
                        .map(|diagnostic| BuildDiagnostic {
                            diagnostic: diagnostic.clone(),
                            path: Some(path.to_path_buf()),
                        })
                        .collect(),
                    display_paths: true,
                },
            }
        } else {
            Self::new(error.to_string())
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.failure {
            Failure::Message(detail) => formatter.write_str(detail),
            Failure::Link(error) => error.fmt(formatter),
            Failure::Diagnostics {
                records,
                display_paths,
            } => {
                for (index, record) in records.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str("\n")?;
                    }
                    let text = match record.path.as_ref().filter(|_| *display_paths) {
                        Some(path) => {
                            fpas_diagnostics::render(&path.to_string_lossy(), &record.diagnostic)
                        }
                        None => fpas_diagnostics::render_without_path(&record.diagnostic),
                    };
                    formatter.write_str(&text)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.link_error()
            .map(|error| error as &dyn std::error::Error)
    }
}

#[cfg(test)]
mod tests;
