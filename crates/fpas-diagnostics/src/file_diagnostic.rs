//! A diagnostic paired with the file it concerns.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::PathBuf;

use crate::{Diagnostic, render, render_without_path};

/// A diagnostic and the source or manifest file it concerns, when known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiagnostic {
    /// Original diagnostic, including its code, source ID and structured details.
    pub diagnostic: Diagnostic,
    /// File named by the producer; `None` when the record concerns no single file.
    pub path: Option<PathBuf>,
}

impl FileDiagnostic {
    /// Pairs a diagnostic with an optional file path.
    #[must_use]
    pub fn new(diagnostic: Diagnostic, path: Option<PathBuf>) -> Self {
        Self { diagnostic, path }
    }
}

impl fmt::Display for FileDiagnostic {
    /// Renders the text form, prefixed by the path when one is known.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match &self.path {
            Some(path) => render(&path.to_string_lossy(), &self.diagnostic),
            None => render_without_path(&self.diagnostic),
        };
        formatter.write_str(&text)
    }
}
