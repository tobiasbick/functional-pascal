//! Project-aware reference discovery for resolved declarations.

use std::path::PathBuf;
use std::sync::Arc;

use super::{NavigationDocument, resolve, token_name};
use crate::{CancellationToken, LanguageServiceError};
use crate::{DocumentSnapshot, DocumentSymbol};
use fpas_diagnostics::SourceSpan;

/// One declaration or usage location for a resolved symbol.
#[derive(Debug, Clone)]
pub struct ReferenceLocation {
    /// Source containing the declaration or usage.
    pub path: PathBuf,
    /// Exact source range occupied by the identifier.
    pub span: SourceSpan,
    /// Whether this location is the defining declaration.
    pub is_declaration: bool,
    /// Exact source snapshot from which `span` was computed.
    pub snapshot: Arc<DocumentSnapshot>,
}

impl PartialEq for ReferenceLocation {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.span == other.span
            && self.is_declaration == other.is_declaration
    }
}

impl Eq for ReferenceLocation {}

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
    cancellation: &CancellationToken,
) -> Result<Vec<ReferenceLocation>, LanguageServiceError> {
    cancellation.check()?;
    let declaration_document = &documents[target.document_index];
    let mut locations = Vec::new();
    if include_declaration {
        locations.push(ReferenceLocation {
            path: declaration_document.path.clone(),
            span: target.symbol.selection_span,
            is_declaration: true,
            snapshot: Arc::clone(&declaration_document.snapshot),
        });
    }

    let candidate_name = target
        .symbol
        .name
        .rsplit('.')
        .next()
        .unwrap_or(&target.symbol.name);
    for (document_index, document) in documents.iter().enumerate() {
        cancellation.check()?;
        for (token_index, token) in document.tokens.iter().enumerate() {
            cancellation.check()?;
            let Some(name) = token_name(document, token_index) else {
                continue;
            };
            if !name.eq_ignore_ascii_case(candidate_name) {
                continue;
            }
            let token_span = token.span.diagnostic_span_or_synthetic();
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
                    snapshot: Arc::clone(&document.snapshot),
                });
            }
        }
    }

    locations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.span.offset().cmp(&right.span.offset()))
            .then_with(|| left.is_declaration.cmp(&right.is_declaration))
    });
    locations.dedup_by(|left, right| left.path == right.path && left.span == right.span);
    Ok(locations)
}

fn same_declaration(left: &ResolvedTarget, right: &ResolvedTarget) -> bool {
    left.document_index == right.document_index
        && left.symbol.selection_span == right.symbol.selection_span
}

fn spans_overlap(left: SourceSpan, right: SourceSpan) -> bool {
    left.offset() < right.end() && right.offset() < left.end()
}
