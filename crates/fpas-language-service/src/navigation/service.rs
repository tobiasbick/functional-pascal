//! Language-service entry points for navigation queries.

use std::path::Path;
use std::sync::Arc;

use super::{
    CompletionCandidate, HoverInfo, NavigationDocument, NavigationResult, complete, resolve,
};
use crate::{
    DocumentSnapshot, DocumentSymbol, DocumentSymbols, LanguageService, LanguageServiceError,
    SymbolLocation, WorkspaceKind,
};

struct NavigationContext {
    snapshot: Arc<DocumentSnapshot>,
    documents: Vec<NavigationDocument>,
    target_index: Option<usize>,
}

impl LanguageService {
    /// Returns hierarchical declarations for the current document snapshot.
    pub fn document_symbols(
        &mut self,
        path: &Path,
    ) -> Result<NavigationResult<Vec<DocumentSymbol>>, LanguageServiceError> {
        self.ensure_source_context(path)?;
        let snapshot = self.snapshot(path)?;
        let value = if self.navigation_allowed(path) {
            DocumentSymbols::from_snapshot(&snapshot).entries().to_vec()
        } else {
            Vec::new()
        };
        Ok(NavigationResult { snapshot, value })
    }

    /// Returns declaration hover information at a UTF-8 byte offset.
    pub fn hover(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Option<HoverInfo>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context.target_index.and_then(|target_index| {
            resolve(&context.documents, target_index, offset).map(|(_, symbol, range)| HoverInfo {
                contents: symbol.detail,
                range,
            })
        });
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }

    /// Returns defining declarations at a UTF-8 byte offset.
    pub fn definitions(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Vec<SymbolLocation>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context
            .target_index
            .and_then(|target_index| resolve(&context.documents, target_index, offset))
            .map(|(document_index, symbol, _)| {
                vec![SymbolLocation {
                    path: context.documents[document_index].path.clone(),
                    symbol,
                }]
            })
            .unwrap_or_default();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }

    /// Returns declarations visible for completion at a UTF-8 byte offset.
    pub fn completions(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Vec<CompletionCandidate>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context
            .target_index
            .map(|target_index| complete(&context.documents, target_index, offset))
            .unwrap_or_default();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }

    fn navigation_context(
        &mut self,
        path: &Path,
    ) -> Result<NavigationContext, LanguageServiceError> {
        self.ensure_source_context(path)?;
        let target = self.snapshot(path)?;
        if !self.navigation_allowed(path) {
            return Ok(NavigationContext {
                snapshot: target,
                documents: Vec::new(),
                target_index: None,
            });
        }
        let project = self.workspace().project_for_source(path).cloned();
        let paths = project
            .as_ref()
            .map(|project| project.all_source_paths())
            .unwrap_or_else(|| vec![path.to_path_buf()]);
        let mut documents = Vec::with_capacity(paths.len());
        for source_path in paths {
            let document = NavigationDocument::new(self.snapshot(&source_path)?);
            if project.as_ref().is_none_or(|project| {
                source_path == path
                    || project.source_visible_from(path, &source_path, &document.owner)
            }) {
                documents.push(document);
            }
        }
        if !documents
            .iter()
            .any(|document| document.path == target.path())
        {
            documents.push(NavigationDocument::new(Arc::clone(&target)));
        }
        let target_index = documents
            .iter()
            .position(|document| document.path == target.path());
        Ok(NavigationContext {
            snapshot: target,
            documents,
            target_index,
        })
    }

    fn navigation_allowed(&self, path: &Path) -> bool {
        matches!(
            self.workspace().kind(),
            WorkspaceKind::Loose | WorkspaceKind::Folder
        ) || self.workspace().project_for_source(path).is_some()
    }
}
