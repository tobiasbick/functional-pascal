//! Source-local namespace declarations for import aliases.
//!
//! See `docs/pascal/program-structure/units.md`.

use fpas_diagnostics::SourceSpan;
use fpas_parser::Import;

use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};

/// Extracts aliases without changing the imported unit's canonical identity.
pub(super) fn alias_symbols(
    owner: &str,
    imports: &[Import],
    scope_span: SourceSpan,
) -> Vec<DocumentSymbol> {
    imports
        .iter()
        .filter_map(|import| {
            let alias = import.alias.as_ref()?;
            Some(DocumentSymbol {
                name: alias.name.clone(),
                qualified_name: format!("{owner}.{}", alias.name),
                kind: SymbolKind::ImportAlias,
                full_span: import.span.diagnostic_span_or_synthetic(),
                selection_span: alias.span.diagnostic_span_or_synthetic(),
                scope_span,
                visible_from: alias.span.offset,
                visibility: SymbolVisibility::Private,
                type_name: None,
                detail: format!("uses {} as {}", import.unit.parts.join("."), alias.name),
                callable: None,
                children: Vec::new(),
            })
        })
        .collect()
}
