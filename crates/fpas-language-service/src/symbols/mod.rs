//! Editor-oriented declaration and workspace symbol indexes.

mod document;
mod index;

pub use document::{DocumentSymbol, DocumentSymbols, SymbolKind};
pub use index::{SymbolLocation, WorkspaceSymbolIndex};
