//! Resolve record constructor aliases to the original fields and visibility owner.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::{NavigationDocument, member_type};
use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};

/// Resolve a type-call target through transparent aliases, guarding cycles and private fields.
pub(crate) fn constructor_record(
    documents: &[NavigationDocument],
    target_index: usize,
    index: usize,
    symbol: DocumentSymbol,
) -> Option<(usize, DocumentSymbol)> {
    let (index, record) = member_type(documents, index, &symbol)?;
    if record.kind != SymbolKind::Type
        || record.callable.is_none()
        || (index != target_index
            && record.children.iter().any(|field| {
                field.kind == SymbolKind::Field && field.visibility == SymbolVisibility::Private
            }))
    {
        return None;
    }
    Some((index, record.clone()))
}
