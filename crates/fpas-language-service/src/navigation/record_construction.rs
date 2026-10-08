//! Resolve record constructor aliases to the original fields and visibility owner.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::{NavigationDocument, find_type};
use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};
use std::collections::HashSet;

/// Resolve a type-call target through transparent aliases, guarding cycles and private fields.
pub(crate) fn constructor_record(
    documents: &[NavigationDocument],
    target_index: usize,
    mut index: usize,
    mut symbol: DocumentSymbol,
) -> Option<(usize, DocumentSymbol)> {
    let mut visited = HashSet::new();
    loop {
        if symbol.kind != SymbolKind::Type
            || !visited.insert(symbol.qualified_name.to_ascii_lowercase())
        {
            return None;
        }
        if symbol.callable.is_some() {
            if index != target_index
                && symbol.children.iter().any(|field| {
                    field.kind == SymbolKind::Field && field.visibility == SymbolVisibility::Private
                })
            {
                return None;
            }
            return Some((index, symbol));
        }
        let name = symbol.type_name.as_deref()?;
        let (next, record) = find_type(documents, index, index, name)?;
        index = next;
        symbol = record.clone();
    }
}
