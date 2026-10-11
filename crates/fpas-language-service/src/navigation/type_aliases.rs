//! Shared transparent type-alias traversal for source-owned members.
//! See `docs/pascal/language/types/records.md`.

use std::collections::HashSet;

use super::{NavigationDocument, find_type};
use crate::{DocumentSymbol, SymbolKind};

/// Follows aliases in their declaring import environment to a record or enum owner.
pub(crate) fn member_type<'a>(
    documents: &'a [NavigationDocument],
    mut index: usize,
    mut symbol: &'a DocumentSymbol,
) -> Option<(usize, &'a DocumentSymbol)> {
    let mut visited = HashSet::new();
    loop {
        if !matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum)
            || !visited.insert((index, symbol.selection_span))
        {
            return None;
        }
        if symbol.callable.is_some() || symbol.kind == SymbolKind::Enum {
            return Some((index, symbol));
        }
        let name = symbol.type_name.as_deref()?;
        (index, symbol) = find_type(documents, index, index, name)?;
    }
}
