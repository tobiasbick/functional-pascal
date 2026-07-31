//! Project-aware reference discovery for resolved declarations.

use std::path::PathBuf;

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Token;

use super::{NavigationDocument, resolve};
use crate::DocumentSymbol;

/// One declaration or usage location for a resolved symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLocation {
    /// Source containing the declaration or usage.
    pub path: PathBuf,
    /// Exact source range occupied by the identifier.
    pub span: SourceSpan,
    /// Whether this location is the defining declaration.
    pub is_declaration: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedTarget {
    pub(crate) document_index: usize,
    pub(crate) symbol: DocumentSymbol,
    pub(crate) occurrence_span: SourceSpan,
}

pub(crate) fn resolve_target(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
) -> Option<ResolvedTarget> {
    let (document_index, symbol, occurrence_span) = resolve(documents, target_index, offset)?;
    Some(ResolvedTarget {
        document_index,
        symbol,
        occurrence_span,
    })
}

pub(crate) fn find_references(
    documents: &[NavigationDocument],
    target: &ResolvedTarget,
    include_declaration: bool,
) -> Vec<ReferenceLocation> {
    let declaration_document = &documents[target.document_index];
    let mut locations = Vec::new();
    if include_declaration {
        locations.push(ReferenceLocation {
            path: declaration_document.path.clone(),
            span: target.symbol.selection_span,
            is_declaration: true,
        });
    }

    let candidate_name = target
        .symbol
        .name
        .rsplit('.')
        .next()
        .unwrap_or(&target.symbol.name);
    for (document_index, document) in documents.iter().enumerate() {
        for token in &document.tokens {
            let Token::Ident(name) = &token.token else {
                continue;
            };
            if !name.eq_ignore_ascii_case(candidate_name) {
                continue;
            }
            let token_span = SourceSpan::from(token.span);
            if document_index == target.document_index
                && spans_overlap(token_span, target.symbol.selection_span)
            {
                continue;
            }
            let Some(resolved) = resolve_target(documents, document_index, token.span.offset)
            else {
                continue;
            };
            if same_declaration(&resolved, target) {
                locations.push(ReferenceLocation {
                    path: document.path.clone(),
                    span: resolved.occurrence_span,
                    is_declaration: false,
                });
            }
        }
    }

    locations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.span.offset.cmp(&right.span.offset))
            .then_with(|| left.is_declaration.cmp(&right.is_declaration))
    });
    locations.dedup_by(|left, right| left.path == right.path && left.span == right.span);
    locations
}

fn same_declaration(left: &ResolvedTarget, right: &ResolvedTarget) -> bool {
    left.document_index == right.document_index
        && left.symbol.selection_span == right.symbol.selection_span
}

fn spans_overlap(left: SourceSpan, right: SourceSpan) -> bool {
    let left_end = left.offset.saturating_add(left.length);
    let right_end = right.offset.saturating_add(right.length);
    left.offset < right_end && right.offset < left_end
}
