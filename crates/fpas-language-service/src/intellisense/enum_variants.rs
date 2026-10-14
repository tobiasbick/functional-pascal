//! Completion spellings for user enum variants always include their visible type.
//! See `docs/pascal/language/types/enums.md`.

use fpas_diagnostics::SourceSpan;

use super::completion::{declaration_candidate, visible_candidates};
use super::{CompletionCandidate, CompletionSource};
use crate::navigation::{NavigationDocument, find_type, member_type};
use crate::{SymbolKind, SymbolVisibility};

/// Offer local and directly imported variants using the required source qualifier.
pub(super) fn qualified_variant_candidates(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
    prefix: &str,
    replacement: SourceSpan,
) -> Vec<CompletionCandidate> {
    let mut types = visible_candidates(documents, target_index, offset)
        .into_iter()
        .filter(|(_, symbol)| matches!(symbol.kind, SymbolKind::Enum | SymbolKind::Type))
        .map(|(index, symbol)| (index, symbol, symbol.name.clone()))
        .collect::<Vec<_>>();
    for (index, document) in documents.iter().enumerate() {
        if index == target_index {
            continue;
        }
        let Some(alias) = documents[target_index].namespace_for(&document.owner) else {
            continue;
        };
        types.extend(document.top_level().iter().filter_map(|symbol| {
            (symbol.visibility == SymbolVisibility::Public
                && matches!(symbol.kind, SymbolKind::Enum | SymbolKind::Type))
            .then(|| (index, symbol, format!("{alias}.{}", symbol.name)))
        }));
    }
    let mut candidates = Vec::new();
    for (index, symbol, qualifier) in types {
        let Some((index, resolved)) =
            find_type(documents, target_index, index, &symbol.qualified_name)
                .and_then(|(index, symbol)| member_type(documents, index, symbol))
        else {
            continue;
        };
        if resolved.kind != SymbolKind::Enum {
            continue;
        }
        for member in resolved.type_members().filter(|member| {
            member.kind == SymbolKind::EnumMember
                && member
                    .name
                    .get(..prefix.len())
                    .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                && (index == target_index || member.visibility == SymbolVisibility::Public)
        }) {
            let mut candidate = declaration_candidate(
                documents,
                index,
                member,
                replacement,
                if index == target_index {
                    CompletionSource::Declaration
                } else {
                    CompletionSource::AutoImport
                },
                0,
                None,
            );
            candidate.insert_text = format!("{qualifier}.{}", member.name);
            candidate.label = candidate.insert_text.clone();
            candidates.push(candidate);
        }
    }
    candidates
}
