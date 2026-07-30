//! Editor-oriented declaration and workspace symbol indexes.

mod document;
mod extract;
mod index;

pub use document::{DocumentSymbol, DocumentSymbols, SymbolKind, SymbolVisibility};
pub use index::{SymbolLocation, WorkspaceSymbolIndex};
