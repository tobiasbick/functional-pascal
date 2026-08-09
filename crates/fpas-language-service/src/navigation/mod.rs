//! Protocol-independent editor navigation queries.

mod document;
mod highlights;
mod references;
mod rename;
mod resolve;
mod selection;
mod service;
mod type_definition;
mod workspace_symbols;

use std::sync::Arc;

use crate::DocumentSnapshot;

pub use workspace_symbols::WORKSPACE_SYMBOL_LIMIT;

pub(crate) use document::NavigationDocument;
pub use highlights::{DocumentHighlight, HighlightKind};
pub use references::ReferenceLocation;
pub(crate) use references::{find_references, resolve_target};
pub use rename::{RenameEdit, RenameError, RenameTarget};
pub(crate) use rename::{prepare_rename, rename_symbol};
pub(crate) use resolve::{find_type, resolve, resolve_qualified, resolve_unqualified};
pub use selection::SelectionRange;

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
    /// Markdown documentation attached to the resolved declaration.
    pub documentation: Option<String>,
    /// Identifier range under the cursor.
    pub range: fpas_diagnostics::SourceSpan,
}
