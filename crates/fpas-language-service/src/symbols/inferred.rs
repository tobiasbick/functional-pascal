//! Checked local types shared by hover, completion, and navigation symbols.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use fpas_sema::{AnalysisMetadata, Ty};

use super::{DocumentSymbol, SymbolKind};

/// Enriches unannotated local symbols with the semantic checker's inferred types.
pub(super) fn apply_binding_types(symbols: &mut [DocumentSymbol], metadata: &AnalysisMetadata) {
    for symbol in symbols {
        let span = symbol.full_span;
        if let Some(ty) = metadata
            .binding_types
            .get(&(span.source_id(), span.offset()))
            && !ty.is_error()
        {
            let keyword = match symbol.kind {
                SymbolKind::Constant => Some("const"),
                SymbolKind::Variable => Some("var"),
                _ => None,
            };
            if let Some(keyword) = keyword {
                symbol.detail = format!("{keyword} {}: {ty}", symbol.name);
                symbol.type_name = match ty {
                    Ty::Record(record) => Some(record.name.clone()),
                    Ty::Enum(enumeration) => Some(enumeration.name.clone()),
                    Ty::Subrange(range) => Some(range.name.clone()),
                    Ty::Distinct(distinct) => Some(match distinct.owner_unit.as_deref() {
                        Some(unit) => {
                            let qualifier = metadata
                                .import_aliases
                                .get(&unit.to_ascii_lowercase())
                                .map_or(unit, String::as_str);
                            format!("{qualifier}.{}", distinct.name)
                        }
                        None => distinct.name.clone(),
                    }),
                    Ty::Named(name) => Some(name.clone()),
                    _ => None,
                };
            }
        }
        apply_binding_types(&mut symbol.children, metadata);
    }
}
