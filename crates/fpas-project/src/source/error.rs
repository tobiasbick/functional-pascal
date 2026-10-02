//! Project failures preserve coded diagnostics until the caller chooses a renderer.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::{Diagnostic, DiagnosticCode, SourceSpan};

/// A project-loading, workspace or unit-graph failure.
///
/// Every failure carries at least one coded diagnostic. Source reading, lexing
/// and parsing failures also retain the path of the failing source; manifest and
/// graph validation failures name their files in the message instead, except
/// `uses` failures inside a unit, which are located at the import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectError {
    path: Option<PathBuf>,
    diagnostics: Vec<Diagnostic>,
}

impl ProjectError {
    /// Creates a positionless validation failure without a help line.
    pub(crate) fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            path: None,
            diagnostics: vec![Diagnostic::error_without_source(code, message, None)],
        }
    }

    /// Attaches actionable help to every record of this failure.
    #[must_use]
    pub(crate) fn with_help(mut self, help: impl Into<String>) -> Self {
        let help = help.into();
        for diagnostic in &mut self.diagnostics {
            diagnostic.help = Some(help.clone());
        }
        self
    }

    /// Places a validation failure at a known range inside one source file.
    ///
    /// The range is omitted when the parser span cannot form a diagnostic span.
    #[must_use]
    pub(crate) fn at_source(mut self, path: &Path, span: fpas_lexer::Span) -> Self {
        let span = SourceSpan::try_from(span).ok();
        for diagnostic in &mut self.diagnostics {
            diagnostic.span = span;
        }
        self.path = Some(path.to_path_buf());
        self
    }

    /// Associates diagnostics produced from one source with its authoritative path.
    pub(crate) fn from_source(path: &Path, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            path: Some(path.to_path_buf()),
            diagnostics,
        }
    }

    /// Returns the coded diagnostics in producer order; never empty.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the source file that produced the diagnostics, even without a position.
    ///
    /// Manifest, workspace and graph validation failures return `None`.
    #[must_use]
    pub fn source_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            let text = match &self.path {
                Some(path) => fpas_diagnostics::render(&path.to_string_lossy(), diagnostic),
                None => fpas_diagnostics::render_without_path(diagnostic),
            };
            formatter.write_str(&text)?;
        }
        Ok(())
    }
}

impl std::error::Error for ProjectError {}
