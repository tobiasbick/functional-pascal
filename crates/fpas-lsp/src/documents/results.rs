//! Snapshot-bound results returned by blocking language-service queries.

use std::path::PathBuf;
use std::sync::Arc;

use fpas_language_service::{
    DocumentSnapshot, ReferenceLocation, RenameEdit, SelectionRange, SemanticCodeAction,
    SymbolLocation,
};
use tower_lsp_server::ls_types::{Diagnostic, Uri};

#[derive(Clone)]
pub(crate) struct SynchronizedDocument {
    pub(crate) path: PathBuf,
    pub(crate) uri: Uri,
    pub(crate) version: i32,
}

pub(crate) struct FormattedDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) text: String,
}

pub(crate) struct DefinitionDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) location: SymbolLocation,
}

pub(crate) struct ReferenceDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) location: ReferenceLocation,
}

pub(crate) struct RenameDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) edit: RenameEdit,
}

pub(crate) struct WorkspaceSymbolDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) location: SymbolLocation,
}

pub(crate) struct SelectionDocument {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) ranges: Vec<SelectionRange>,
}

pub(crate) struct CodeActionResult {
    pub(crate) snapshot: Arc<DocumentSnapshot>,
    pub(crate) actions: Vec<(SemanticCodeAction, Diagnostic)>,
}
