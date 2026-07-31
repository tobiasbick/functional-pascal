//! Language-service entry points for navigation queries.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use super::{
    CompletionCandidate, HoverInfo, NavigationDocument, NavigationResult, ReferenceLocation,
    RenameEdit, RenameError, RenameTarget, complete, find_references, prepare_rename,
    rename_symbol, resolve, resolve_target,
};
use crate::{
    CancellationToken, DocumentSnapshot, DocumentSymbol, DocumentSymbols, LanguageService,
    LanguageServiceError, SymbolLocation, WorkspaceKind,
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

    /// Returns project references for the declaration resolved at a UTF-8 byte offset.
    pub fn references(
        &mut self,
        path: &Path,
        offset: usize,
        include_declaration: bool,
    ) -> Result<NavigationResult<Vec<ReferenceLocation>>, LanguageServiceError> {
        self.references_with_cancellation(
            path,
            offset,
            include_declaration,
            &CancellationToken::new(),
        )
    }

    /// Returns project references while observing a cooperative cancellation signal.
    pub fn references_with_cancellation(
        &mut self,
        path: &Path,
        offset: usize,
        include_declaration: bool,
        cancellation: &CancellationToken,
    ) -> Result<NavigationResult<Vec<ReferenceLocation>>, LanguageServiceError> {
        let context = self.reference_navigation_context(path, offset, cancellation)?;
        let value = context
            .target_index
            .and_then(|target_index| resolve_target(&context.documents, target_index, offset))
            .map(|target| {
                find_references(
                    &context.documents,
                    &target,
                    include_declaration,
                    cancellation,
                )
            })
            .transpose()?
            .unwrap_or_default();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }

    /// Returns the renameable identifier range and source spelling at a UTF-8 byte offset.
    pub fn prepare_rename(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Option<RenameTarget>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context.target_index.and_then(|target_index| {
            prepare_rename(
                &context.documents,
                target_index,
                offset,
                self.workspace().root(),
            )
        });
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }

    /// Returns validated project edits that rename the declaration at a UTF-8 byte offset.
    ///
    /// # Errors
    ///
    /// Returns [`RenameError`] when source loading fails, the selected declaration is not safely
    /// renameable, the replacement is invalid, or it conflicts in the declaration scope.
    pub fn rename(
        &mut self,
        path: &Path,
        offset: usize,
        new_name: &str,
    ) -> Result<NavigationResult<Vec<RenameEdit>>, RenameError> {
        self.rename_with_cancellation(path, offset, new_name, &CancellationToken::new())
    }

    /// Returns validated rename edits while observing a cooperative cancellation signal.
    pub fn rename_with_cancellation(
        &mut self,
        path: &Path,
        offset: usize,
        new_name: &str,
        cancellation: &CancellationToken,
    ) -> Result<NavigationResult<Vec<RenameEdit>>, RenameError> {
        let context = self.reference_navigation_context(path, offset, cancellation)?;
        let target_index = context.target_index.ok_or(RenameError::NoSymbol)?;
        let value = rename_symbol(
            &context.documents,
            target_index,
            offset,
            self.workspace().root(),
            new_name,
            cancellation,
        )?;
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
        let project = self.analysis_project_for(path, target.compilation_unit());
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

    fn reference_navigation_context(
        &mut self,
        path: &Path,
        offset: usize,
        cancellation: &CancellationToken,
    ) -> Result<NavigationContext, LanguageServiceError> {
        cancellation.check()?;
        let initial = self.navigation_context(path)?;
        let Some(selected_index) = initial.target_index else {
            return Ok(initial);
        };
        let Some(target) = resolve_target(&initial.documents, selected_index, offset) else {
            return Ok(initial);
        };
        let declaration_path = initial.documents[target.document_index].path.clone();
        let declaration_owner = initial.documents[target.document_index].owner.clone();
        let mut paths = initial
            .documents
            .iter()
            .map(|document| document.path.clone())
            .collect::<BTreeSet<_>>();
        for project in self
            .workspace()
            .projects()
            .iter()
            .filter(|project| project.contains_source(&declaration_path))
        {
            cancellation.check()?;
            for source_path in project.all_source_paths() {
                if source_path == declaration_path
                    || project.source_visible_from(
                        &source_path,
                        &declaration_path,
                        &declaration_owner,
                    )
                {
                    paths.insert(source_path);
                }
            }
        }

        let mut documents = Vec::with_capacity(paths.len());
        for source_path in paths {
            cancellation.check()?;
            documents.push(NavigationDocument::new(self.snapshot(&source_path)?));
        }
        let target_index = documents
            .iter()
            .position(|document| document.path == initial.snapshot.path());
        Ok(NavigationContext {
            snapshot: initial.snapshot,
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
