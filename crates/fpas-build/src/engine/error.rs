//! Build failures retain diagnostic records until an output boundary renders them.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::Path;

use fpas_diagnostics::{Diagnostic, DiagnosticCode, FileDiagnostic};
use fpas_linker::LinkError;

/// Incremental build failure with coded project, compiler, parser or linker records.
///
/// Every failure carries at least one record. Linker failures additionally keep the
/// typed [`LinkError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    records: Vec<FileDiagnostic>,
    display_paths: bool,
    link: Option<Box<LinkError>>,
}

impl BuildError {
    /// Creates a positionless build failure without a help line.
    pub(crate) fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::from_records(vec![FileDiagnostic {
            diagnostic: Diagnostic::error_without_source(code, message, None),
            path: None,
        }])
    }

    /// Attaches actionable help to every record of this failure.
    #[must_use]
    pub(crate) fn with_help(mut self, help: impl Into<String>) -> Self {
        let help = help.into();
        for record in &mut self.records {
            record.diagnostic.help = Some(help.clone());
        }
        self
    }

    /// Attributes a positionless failure to the source file it concerns.
    #[must_use]
    pub(crate) fn in_source(mut self, path: &Path) -> Self {
        for record in &mut self.records {
            record.path = Some(path.to_path_buf());
        }
        self
    }

    /// Retains producer records and associates only matching source IDs with a path.
    pub(crate) fn from_diagnostics(
        diagnostics: Vec<Diagnostic>,
        source: Option<(u32, &Path)>,
    ) -> Self {
        Self::from_records(
            diagnostics
                .into_iter()
                .map(|diagnostic| {
                    let path = source
                        .filter(|(id, _)| {
                            diagnostic.span.is_some_and(|span| span.source_id() == *id)
                        })
                        .map(|(_, path)| path.to_path_buf());
                    FileDiagnostic { diagnostic, path }
                })
                .collect(),
        )
    }

    /// Retains root compiler records while leaving text file context to the caller.
    pub(crate) fn from_program_diagnostics(
        diagnostics: Vec<Diagnostic>,
        path: Option<&Path>,
    ) -> Self {
        let mut error = Self::from_diagnostics(diagnostics, path.map(|path| (0, path)));
        // Program callers already supply the main-file context in text output.
        error.display_paths = false;
        error
    }

    fn from_records(records: Vec<FileDiagnostic>) -> Self {
        Self {
            records,
            display_paths: true,
            link: None,
        }
    }

    /// Returns the coded records in producer order; never empty.
    ///
    /// No records are recovered by parsing formatted text.
    #[must_use]
    pub fn diagnostics(&self) -> &[FileDiagnostic] {
        &self.records
    }

    /// Returns the original typed linker failure, when linking failed.
    #[must_use]
    pub fn link_error(&self) -> Option<&LinkError> {
        self.link.as_deref()
    }
}

impl From<LinkError> for BuildError {
    fn from(error: LinkError) -> Self {
        let mut failure = Self::new(error.code(), error.to_string());
        failure.link = Some(Box::new(error));
        failure
    }
}

impl From<fpas_project::ProjectError> for BuildError {
    fn from(error: fpas_project::ProjectError) -> Self {
        let path = error.source_path();
        Self::from_records(
            error
                .diagnostics()
                .iter()
                .map(|diagnostic| FileDiagnostic {
                    diagnostic: diagnostic.clone(),
                    path: path.map(Path::to_path_buf),
                })
                .collect(),
        )
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, record) in self.records.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            if self.display_paths {
                write!(formatter, "{record}")?;
            } else {
                formatter.write_str(&fpas_diagnostics::render_without_path(&record.diagnostic))?;
            }
        }
        Ok(())
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
