//! Import-alias reservations shared by local and namespace renames.
//!
//! See `docs/pascal/program-structure/units.md`.

use super::RenameError;
use crate::SymbolKind;
use crate::navigation::{NavigationDocument, references::ResolvedTarget};

/// Rejects shadowing an alias, and alias names reserved by this source's namespaces.
pub(super) fn reject_alias_conflicts(
    documents: &[NavigationDocument],
    target: &ResolvedTarget,
    new_name: &str,
) -> Result<(), RenameError> {
    if !lexical_kind(target.symbol.kind) {
        return Ok(());
    }
    let document = &documents[target.document_index];
    let renaming_alias = target.symbol.kind == SymbolKind::ImportAlias;
    let conflict = document.all_symbols().into_iter().any(|symbol| {
        symbol.selection_span != target.symbol.selection_span
            && symbol.name.eq_ignore_ascii_case(new_name)
            && lexical_kind(symbol.kind)
            && (renaming_alias || symbol.kind == SymbolKind::ImportAlias)
    });
    let reserved = renaming_alias
        && (["Std", "integer", "real", "boolean", "string"]
            .iter()
            .any(|name| name.eq_ignore_ascii_case(new_name))
            || document
                .owner
                .split('.')
                .next()
                .is_some_and(|root| root.eq_ignore_ascii_case(new_name))
            || document.uses.iter().any(|import| {
                import
                    .unit
                    .parts
                    .first()
                    .is_some_and(|root| root.eq_ignore_ascii_case(new_name))
            }));
    if conflict || reserved {
        Err(RenameError::Conflict {
            name: new_name.to_owned(),
        })
    } else {
        Ok(())
    }
}

fn lexical_kind(kind: SymbolKind) -> bool {
    !matches!(
        kind,
        SymbolKind::Program
            | SymbolKind::Unit
            | SymbolKind::Field
            | SymbolKind::Property
            | SymbolKind::Event
            | SymbolKind::Method
            | SymbolKind::EnumMember
    )
}
