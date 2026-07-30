//! Protocol-independent editor navigation queries.

mod document;
mod resolve;
mod service;

use std::sync::Arc;

use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind};

pub(crate) use document::NavigationDocument;
pub(crate) use resolve::{complete, resolve};

/// A query result tied to the exact immutable source snapshot used for positions.
#[derive(Debug, Clone)]
pub struct NavigationResult<T> {
    /// Exact source snapshot used by the query.
    pub snapshot: Arc<DocumentSnapshot>,
    /// Protocol-independent query value.
    pub value: T,
}

/// Hover text and the source range it describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverInfo {
    /// Compact Functional Pascal declaration text.
    pub contents: String,
    /// Identifier range under the cursor.
    pub range: fpas_diagnostics::SourceSpan,
}

/// One completion entry derived from a currently visible declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    /// Source spelling inserted by the editor.
    pub label: String,
    /// Declaration category.
    pub kind: SymbolKind,
    /// Compact source-level declaration detail.
    pub detail: String,
    /// Owner-qualified identity used to distinguish equal labels.
    pub qualified_name: String,
}

impl From<&DocumentSymbol> for CompletionCandidate {
    fn from(symbol: &DocumentSymbol) -> Self {
        Self {
            label: symbol.name.clone(),
            kind: symbol.kind,
            detail: symbol.detail.clone(),
            qualified_name: symbol.qualified_name.clone(),
        }
    }
}
