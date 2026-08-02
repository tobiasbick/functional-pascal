//! Parsed immutable source snapshots.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_parser::{CompilationUnit, ParseDiagnostic, parse_compilation_unit};

use super::{LineIndex, normalized_path};

/// Identity of the source revision represented by a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceVersion {
    /// Monotonic version supplied by an editor client.
    Editor(i64),
    /// Monotonic revision assigned when disk contents change.
    Disk(u64),
}

/// Immutable parsed state for one exact source revision.
#[derive(Debug)]
pub struct DocumentSnapshot {
    path: PathBuf,
    version: SourceVersion,
    revision: u64,
    source: Arc<str>,
    line_index: LineIndex,
    compilation_unit: Arc<CompilationUnit>,
    parse_diagnostics: Arc<[ParseDiagnostic]>,
}

impl DocumentSnapshot {
    pub(crate) fn parse(
        path: &Path,
        version: SourceVersion,
        revision: u64,
        source: Arc<str>,
    ) -> Self {
        let line_index = LineIndex::new(Arc::clone(&source));
        let (compilation_unit, parse_diagnostics) = parse_compilation_unit(&source);
        Self {
            path: normalized_path(path),
            version,
            revision,
            source,
            line_index,
            compilation_unit: Arc::new(compilation_unit),
            parse_diagnostics: parse_diagnostics.into(),
        }
    }

    /// Returns the normalized local source path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the exact source revision represented by this snapshot.
    #[must_use]
    pub fn version(&self) -> SourceVersion {
        self.version
    }

    /// Returns the store-owned identity of this exact snapshot lifetime.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the immutable UTF-8 source text.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the reusable line index for this source.
    #[must_use]
    pub fn line_index(&self) -> &LineIndex {
        &self.line_index
    }

    /// Returns the recovered compilation-unit AST for this exact source allocation.
    #[must_use]
    pub fn compilation_unit(&self) -> &CompilationUnit {
        &self.compilation_unit
    }

    /// Returns lexer and parser diagnostics in source order.
    #[must_use]
    pub fn parse_diagnostics(&self) -> &[ParseDiagnostic] {
        &self.parse_diagnostics
    }

    /// Returns whether parsing produced an error-severity diagnostic.
    #[must_use]
    pub fn has_parse_errors(&self) -> bool {
        self.parse_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.as_diagnostic().is_error())
    }
}
