//! Editor-oriented declaration and workspace symbol indexes.

mod document;
mod extract;
mod index;
mod intrinsic_api;

pub use document::{
    CallableSignature, DocumentSymbol, DocumentSymbols, SymbolKind, SymbolVisibility,
};
pub use index::{SymbolLocation, WorkspaceSymbolIndex};
