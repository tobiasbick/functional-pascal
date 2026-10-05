//! Project failures preserve coded diagnostics until the caller chooses a renderer.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::{Diagnostic, DiagnosticCode, SourceSpan};

/// A project-loading, workspace or unit-graph failure.
///
/// Every failure carries at least one coded diagnostic. Source reading, lexing
/// and parsing failures retain the failing source path. Manifest failures retain
/// the owning manifest path without inventing positions. Graph failures may
/// concern several files; `uses` failures are located at the importing unit.
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

    /// Attributes a pathless failure to its owning file, preserving existing source paths.
    #[must_use]
    pub(crate) fn in_file(mut self, path: &Path) -> Self {
        self.path.get_or_insert_with(|| path.to_path_buf());
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

    /// Returns the source or manifest file responsible for the diagnostics, even without a position.
    ///
    /// Failures without an authoritative file, such as multi-file graph failures, return `None`.
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
