//! Collision-safe symbol lookup across documents.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{DocumentSymbol, DocumentSymbols};
use crate::document::normalized_path;

/// One symbol paired with its defining source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolLocation {
    /// Normalized defining source path.
    pub path: PathBuf,
    /// Declaration metadata within the source.
    pub symbol: DocumentSymbol,
}

/// Workspace declaration index that preserves ambiguous short-name candidates.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceSymbolIndex {
    documents: HashMap<PathBuf, DocumentSymbols>,
    qualified: HashMap<String, Vec<SymbolLocation>>,
    unqualified: HashMap<String, Vec<SymbolLocation>>,
}

impl WorkspaceSymbolIndex {
    /// Creates an empty workspace symbol index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces every indexed declaration for one source document.
    pub fn replace_document(&mut self, path: &Path, symbols: DocumentSymbols) {
        self.documents.insert(normalized_path(path), symbols);
        self.rebuild();
    }

    /// Removes one document and all declarations it contributed.
    pub fn remove_document(&mut self, path: &Path) {
        self.documents.remove(&normalized_path(path));
        self.rebuild();
    }

    /// Finds every declaration matching a case-insensitive owner-qualified name.
    #[must_use]
    pub fn find_qualified(&self, name: &str) -> &[SymbolLocation] {
        self.qualified
            .get(&canonical_name(name))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Finds every declaration matching a case-insensitive short name.
    ///
    /// Multiple results are intentional when distinct units declare the same unqualified name.
    #[must_use]
    pub fn find_unqualified(&self, name: &str) -> &[SymbolLocation] {
        self.unqualified
            .get(&canonical_name(name))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Returns the number of indexed source documents.
    #[must_use]
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    fn rebuild(&mut self) {
        self.qualified.clear();
        self.unqualified.clear();
        for (path, document) in &self.documents {
            for symbol in document.entries() {
                let location = SymbolLocation {
                    path: path.clone(),
                    symbol: symbol.clone(),
                };
                self.qualified
                    .entry(canonical_name(&symbol.qualified_name))
                    .or_default()
                    .push(location.clone());
                self.unqualified
                    .entry(canonical_name(&symbol.name))
                    .or_default()
                    .push(location);
            }
        }
        for locations in self
            .qualified
            .values_mut()
            .chain(self.unqualified.values_mut())
        {
            locations.sort_by(|left, right| {
                left.path
                    .cmp(&right.path)
                    .then_with(|| left.symbol.qualified_name.cmp(&right.symbol.qualified_name))
            });
        }
    }
}

fn canonical_name(name: &str) -> String {
    name.to_ascii_lowercase()
}
