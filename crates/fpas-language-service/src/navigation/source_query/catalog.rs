//! Interactive discovery shares bounded ranking but never claims checked declaration identity.

use super::{
    DeclarationOrigin, QueryPolicy, SourceCandidate, SourceFindResult, SourceQueryError,
    SourceQueryTarget, SourceSession,
    collector::{Collector, DeclarationKey, MatchKey},
};
use crate::{
    CancellationToken, DocumentSymbol, LanguageService, LanguageServiceError, identity_path,
};
use std::{collections::BTreeSet, sync::Arc};

impl LanguageService {
    pub(crate) fn catalog_declarations(
        &mut self,
        query: &str,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceFindResult, LanguageServiceError> {
        let query = query.trim().to_ascii_lowercase();
        let mut collector = Collector::new(limit);
        for path in self.catalog_source_paths() {
            cancellation.check()?;
            let snapshot = self.snapshot(&path)?;
            let symbols = if self.is_editor_api_source(&path) {
                crate::DocumentSymbols::from_editor_snapshot(&snapshot)
            } else {
                crate::DocumentSymbols::from_snapshot(&snapshot)
            };
            let file =
                identity_path(&path).map_err(|e| LanguageServiceError::source_read(&path, e))?;
            let owner = self
                .workspace()
                .project_for_source(&path)
                .map(|p| identity_path(p.manifest_path()))
                .transpose()
                .map_err(|e| LanguageServiceError::source_read(&path, e))?;
            visit(symbols.entries(), &mut |symbol| {
                if let Some(key) = MatchKey::new(
                    &query,
                    false,
                    DeclarationKey {
                        name: &symbol.name,
                        qualified: &symbol.qualified_name,
                        owner: owner.as_deref().unwrap_or(""),
                        file: &file,
                        start: symbol.selection_span.offset(),
                        kind: &format!("{:?}", symbol.kind),
                    },
                ) {
                    collector.offer(key, || {
                        let mut shallow = symbol.clone();
                        shallow.children.clear();
                        SourceCandidate {
                            target: SourceQueryTarget {
                                name: symbol.name.clone(),
                                qualified_name: symbol.qualified_name.clone(),
                                kind: Some(symbol.kind),
                                symbol: Some(shallow),
                                snapshot: Some(Arc::clone(&snapshot)),
                                target_ref: None,
                                reference_unavailable: Some("editor_catalog"),
                            },
                            owner: owner.clone(),
                            contexts: owner.clone().into_iter().collect(),
                            origin: DeclarationOrigin::EditorCatalog,
                            checked: false,
                            detail: symbol.detail.clone(),
                            documentation: crate::documentation::preceding_documentation(
                                snapshot.source(),
                                symbol.full_span.offset(),
                            ),
                        }
                    });
                }
            });
        }
        let (symbols, total) = collector.finish();
        let reasons = if total > symbols.len() {
            BTreeSet::from(["result_limit"])
        } else {
            BTreeSet::new()
        };
        Ok(SourceFindResult {
            symbols,
            known_total: Some(total),
            reasons,
            diagnostics: Vec::new(),
        })
    }
}

impl SourceSession {
    /// Searches an explicitly selected editor catalogue, including empty text and loose buffers.
    pub fn find_catalog(
        &mut self,
        query: &str,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceFindResult, SourceQueryError> {
        if self.context().policy != QueryPolicy::EditorCatalog || !(1..=10000).contains(&limit) {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Catalogue search requires editor_catalog policy and a limit between 1 and 10000.",
            ));
        }
        let mut result = self
            .service()
            .catalog_declarations(query, limit, cancellation)?;
        if !self.verify()? {
            result.reasons.insert("source_changed");
            result.known_total = None;
        }
        Ok(result)
    }
}

fn visit(symbols: &[DocumentSymbol], action: &mut impl FnMut(&DocumentSymbol)) {
    for symbol in symbols {
        action(symbol);
        visit(&symbol.children, action);
    }
}
