//! Protocol-independent completion and signature assistance.

mod auto_import;
mod completion;
mod context;
mod signature_help;

use std::path::PathBuf;

use fpas_diagnostics::SourceSpan;

use crate::{CallableSignature, SymbolKind};

/// Editor category for semantic and syntax completion items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// A declaration category from the symbol model.
    Symbol(SymbolKind),
    /// A Functional Pascal keyword.
    Keyword,
}

/// Origin of an editor completion suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionSource {
    /// A declaration already visible at the cursor.
    Declaration,
    /// A syntax keyword appropriate at the cursor.
    Keyword,
    /// A unique public declaration requiring one unit import.
    AutoImport,
}

/// One source edit attached to a completion item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEdit {
    /// Source range replaced by the edit.
    pub span: SourceSpan,
    /// Replacement text.
    pub new_text: String,
}

/// Stable declaration identity used for lazy completion documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionDocumentation {
    /// Source containing the declaration.
    pub path: PathBuf,
    /// Start offset of the complete declaration.
    pub declaration_offset: usize,
    /// Store-owned identity of the exact declaration snapshot.
    pub source_revision: u64,
    /// Owner-qualified declaration name used to reject offset reuse.
    pub qualified_name: String,
}

/// One completion entry grounded in a declaration or parser keyword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    /// Source spelling displayed by the editor.
    pub label: String,
    /// Editor-facing category.
    pub kind: CompletionKind,
    /// Compact source-level declaration detail.
    pub detail: String,
    /// Owner shown beside equal labels.
    pub owner: Option<String>,
    /// Stable owner-qualified identity.
    pub qualified_name: String,
    /// Deterministic editor sort key.
    pub sort_text: String,
    /// Text used by editor-side filtering.
    pub filter_text: String,
    /// Text inserted in place of [`Self::replacement_span`].
    pub insert_text: String,
    /// Exact identifier fragment replaced on acceptance.
    pub replacement_span: SourceSpan,
    /// Suggestion origin.
    pub source: CompletionSource,
    /// Optional declaration identity resolved only after item selection.
    pub documentation: Option<CompletionDocumentation>,
    /// Optional same-document import edit.
    pub additional_edit: Option<CompletionEdit>,
}

/// Signature information for the innermost call containing the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelp {
    /// Resolved callable declaration shape.
    pub signature: CallableSignature,
    /// Markdown attached to the callable declaration.
    pub documentation: Option<String>,
    /// Markdown attached to each explicit parameter in declaration order.
    pub parameter_documentation: Vec<Option<String>>,
    /// Zero-based explicit argument selected by the cursor.
    pub active_parameter: Option<usize>,
}
