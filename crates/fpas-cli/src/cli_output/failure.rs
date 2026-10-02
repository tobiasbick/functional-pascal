//! Coded command failures collected before the selected diagnostic format renders them.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::{Diagnostic, DiagnosticCode, FileDiagnostic};

/// One or more coded records that make a CLI command fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliFailure {
    records: Vec<FileDiagnostic>,
}

impl CliFailure {
    /// Creates a positionless failure that concerns no single file.
    pub(crate) fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            records: vec![FileDiagnostic::new(
                Diagnostic::error_without_source(code, message, None),
                None,
            )],
        }
    }

    /// Codes a CLI-formatted message whose optional hint follows a `\n  help: ` line.
    pub(crate) fn from_message(code: DiagnosticCode, text: &str) -> Self {
        match text.split_once("\n  help: ") {
            Some((message, help)) => Self::new(code, message).with_help(help),
            None => Self::new(code, text),
        }
    }

    /// Attaches actionable help to every record.
    #[must_use]
    pub(crate) fn with_help(mut self, help: impl Into<String>) -> Self {
        let help = help.into();
        for record in &mut self.records {
            record.diagnostic.help = Some(help.clone());
        }
        self
    }

    /// Attaches structured expected/found values to every record.
    #[must_use]
    pub(crate) fn with_expected_found(
        mut self,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        let (expected, found) = (expected.into(), found.into());
        for record in &mut self.records {
            record.diagnostic = record
                .diagnostic
                .clone()
                .with_expected_found(expected.clone(), found.clone());
        }
        self
    }

    /// Attributes every record without a path to `path`.
    #[must_use]
    pub(crate) fn in_file(mut self, path: &Path) -> Self {
        for record in &mut self.records {
            record.path.get_or_insert_with(|| path.to_path_buf());
        }
        self
    }

    /// Wraps compiler or parser diagnostics of one source file.
    pub(crate) fn from_diagnostics(path: &Path, diagnostics: &[Diagnostic]) -> Self {
        Self {
            records: diagnostics
                .iter()
                .map(|diagnostic| FileDiagnostic::new(diagnostic.clone(), Some(path.to_path_buf())))
                .collect(),
        }
    }

    /// Converts a build failure, resolving pathless spans through the graph source table.
    pub(crate) fn from_build(error: &fpas_build::BuildError, source_paths: &[PathBuf]) -> Self {
        Self {
            records: error
                .diagnostics()
                .iter()
                .map(|record| {
                    let path = record.path.clone().or_else(|| {
                        let index = usize::try_from(record.diagnostic.span?.source_id()).ok()?;
                        source_paths.get(index).cloned()
                    });
                    FileDiagnostic::new(record.diagnostic.clone(), path)
                })
                .collect(),
        }
    }

    /// Replaces record paths equal to `portable[i]` with the matching `actual[i]`.
    #[must_use]
    pub(crate) fn with_actual_paths(mut self, portable: &[String], actual: &[PathBuf]) -> Self {
        for record in &mut self.records {
            let Some(path) = &record.path else {
                continue;
            };
            if let Some(index) = portable.iter().position(|name| Path::new(name) == path)
                && let Some(actual) = actual.get(index)
            {
                record.path = Some(actual.clone());
            }
        }
        self
    }

    /// Returns the records in producer order; never empty.
    pub(crate) fn records(&self) -> &[FileDiagnostic] {
        &self.records
    }
}

impl From<fpas_project::ProjectError> for CliFailure {
    fn from(error: fpas_project::ProjectError) -> Self {
        let path = error.source_path().map(Path::to_path_buf);
        Self {
            records: error
                .diagnostics()
                .iter()
                .map(|diagnostic| FileDiagnostic::new(diagnostic.clone(), path.clone()))
                .collect(),
        }
    }
}

impl From<fpas_build::BuildError> for CliFailure {
    fn from(error: fpas_build::BuildError) -> Self {
        Self::from_build(&error, &[])
    }
}

impl fmt::Display for CliFailure {
    /// Renders the text form, one record per line group.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, record) in self.records.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            write!(formatter, "{record}")?;
        }
        Ok(())
    }
}
