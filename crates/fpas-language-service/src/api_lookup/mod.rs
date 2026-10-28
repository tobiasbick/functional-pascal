//! Command-line API lookup over built-in operations, `Std.*` units and project declarations.
//!
//! **Documentation:** `docs/pascal/program-structure/cli.md#looking-up-declarations`

mod declarations;
mod native;
pub(crate) use native::qualified_name as native_qualified_name;
mod suggestions;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{DocumentSnapshot, DocumentSymbols, LanguageService, LanguageServiceError, SymbolKind};

/// Category of one API lookup match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiEntryKind {
    /// Operation on a value of a built-in type, such as `Text.TrimLeft()`.
    Operation,
    /// Factory written on a built-in type name, such as `string.Chr(65)`.
    Factory,
    /// Source or generated declaration.
    Declaration(SymbolKind),
}

/// One-based source position of a declaration; the column counts Unicode scalars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiLocation {
    /// Normalized defining source path.
    pub path: PathBuf,
    /// One-based line.
    pub line: usize,
    /// One-based Unicode-scalar column.
    pub column: usize,
}

/// One declaration listed by a unit match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiMember {
    /// Declaration name without the unit prefix.
    pub name: String,
    /// Declaration signature as shown on hover.
    pub signature: String,
    /// First line of the declaration's documentation comment.
    pub summary: Option<String>,
}

/// One declaration or built-in operation matching a lookup query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiEntry {
    /// Qualified name, such as `Std.Io.Fs.ReadText` or `string.TrimLeft`.
    pub name: String,
    /// Match category.
    pub kind: ApiEntryKind,
    /// Signature as shown on hover.
    pub signature: String,
    /// Markdown documentation as shown on hover.
    pub documentation: Option<String>,
    /// Repository-relative handbook page, when one is known.
    pub handbook: Option<String>,
    /// Declaring source position; `None` for built-in operations.
    pub location: Option<ApiLocation>,
    /// Public declarations of a matched unit, in source order.
    pub members: Vec<ApiMember>,
}

/// Result of one API lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiLookup {
    /// The query as written, without surrounding whitespace.
    pub query: String,
    /// Every match: built-in operations first, then declarations by qualified name.
    pub matches: Vec<ApiEntry>,
    /// Up to five closest names when nothing matches.
    pub suggestions: Vec<String>,
}

/// Searchable snapshot of every declaration and built-in operation known to a service.
pub struct ApiCatalog {
    candidates: Vec<declarations::Candidate>,
    snapshots: HashMap<PathBuf, Arc<DocumentSnapshot>>,
}

impl ApiCatalog {
    /// Looks up a case-insensitive name the way hover describes it.
    ///
    /// A name matches a declaration when it equals the qualified name or one of its dotted
    /// suffixes, so `Std.Io.Fs.ReadText`, `Fs.ReadText` and `ReadText` all find the same routine.
    /// `string.TrimLeft` and `TrimLeft` find built-in operations.
    #[must_use]
    pub fn lookup(&self, query: &str) -> ApiLookup {
        let query = query.trim();
        let mut matches = native::matches(query);
        matches.extend(declarations::matches(
            &self.candidates,
            &self.snapshots,
            query,
        ));
        let suggestions = if matches.is_empty() {
            suggestions::closest(
                query,
                self.candidates
                    .iter()
                    .map(|candidate| candidate.symbol.qualified_name.clone())
                    .chain(native::names()),
            )
        } else {
            Vec::new()
        };
        ApiLookup {
            query: query.to_owned(),
            matches,
            suggestions,
        }
    }
}

impl LanguageService {
    /// Builds the lookup catalog from the loaded workspace and standard library.
    ///
    /// Source-library units are included also when no project is loaded. Nothing is compiled
    /// or written; sources with errors contribute their recovered declarations.
    /// **Documentation:** `docs/pascal/program-structure/cli.md#looking-up-declarations`
    ///
    /// # Errors
    /// Returns source-loading failures of indexed documents.
    pub fn api_catalog(&mut self) -> Result<ApiCatalog, LanguageServiceError> {
        let mut index = self.workspace_symbol_index()?;
        let missing = self
            .standard_library_source_paths()
            .into_iter()
            .filter(|path| !index.contains_document(path))
            .map(|path| {
                let snapshot = self.snapshot(&path)?;
                Ok((path, DocumentSymbols::from_snapshot(&snapshot)))
            })
            .collect::<Result<Vec<_>, LanguageServiceError>>()?;
        index.replace_documents(missing);
        let candidates = declarations::candidates(self, &index);
        let mut snapshots = HashMap::new();
        for candidate in &candidates {
            if !snapshots.contains_key(&candidate.path) {
                snapshots.insert(candidate.path.clone(), self.snapshot(&candidate.path)?);
            }
        }
        Ok(ApiCatalog {
            candidates,
            snapshots,
        })
    }
}

#[cfg(test)]
mod tests;
