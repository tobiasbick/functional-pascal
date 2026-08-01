//! Editor-oriented Functional Pascal source snapshots and analysis.
//!
//! This crate owns no editor protocol types and performs no compilation, execution, or
//! compiled-unit writes. It composes the parser, semantic analyzer, formatter, and project loader
//! into reusable immutable results for a later language-server transport.

#![deny(missing_docs)]

mod analysis;
mod cancellation;
mod diagnostics;
mod document;
mod error;
mod formatting;
mod intellisense;
mod navigation;
mod semantic_tools;
mod symbols;
mod workspace;

pub use analysis::{DocumentAnalysis, LanguageService, SemanticAnalysis};
pub use cancellation::CancellationToken;
pub use diagnostics::diagnostics_for_document;
pub use document::{
    DocumentSnapshot, DocumentStore, LineIndex, SourceVersion, TextPosition, TextRange,
};
pub use error::LanguageServiceError;
pub use formatting::format_document;
pub use intellisense::{
    CompletionCandidate, CompletionDocumentation, CompletionEdit, CompletionKind, CompletionSource,
    SignatureHelp,
};
pub use navigation::{
    DocumentHighlight, HighlightKind, HoverInfo, NavigationResult, ReferenceLocation, RenameEdit,
    RenameError, RenameTarget, SelectionRange, WORKSPACE_SYMBOL_LIMIT,
};
pub use semantic_tools::{
    DiagnosticIdentity, SemanticCodeAction, SemanticEdit, SemanticToken, SemanticTokenKind,
    SemanticTokenModifiers,
};
pub use symbols::{
    CallableSignature, DocumentSymbol, DocumentSymbols, SymbolKind, SymbolLocation,
    SymbolVisibility, WorkspaceSymbolIndex,
};
pub use workspace::{ProjectContext, WorkspaceContext, WorkspaceIssue, WorkspaceKind};
