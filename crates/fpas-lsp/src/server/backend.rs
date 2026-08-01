//! `tower-lsp-server` backend for lifecycle, documents, diagnostics, and formatting.

use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse};
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentFormattingParams, DocumentHighlight,
    DocumentHighlightParams, DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverParams, InitializeParams, InitializeResult,
    InitializedParams, Location, PrepareRenameResponse, ReferenceParams, RenameParams,
    SelectionRange, SelectionRangeParams, SignatureHelp, SignatureHelpParams,
    TextDocumentPositionParams, TextEdit, Uri, WorkspaceEdit, WorkspaceSymbolParams,
    WorkspaceSymbolResponse,
};
use tower_lsp_server::{Client, LanguageServer};

use crate::capabilities;
use crate::convert::file_uri_to_path;
use crate::diagnostics::DiagnosticPublisher;
use crate::documents::{SynchronizedDocument, SynchronizedDocuments};
use crate::formatting::whole_document_edit;
use crate::server::initialization::InitializationPaths;

/// Functional Pascal LSP backend with full-text synchronized documents.
pub struct Backend {
    pub(super) documents: Arc<SynchronizedDocuments>,
    diagnostics: DiagnosticPublisher,
}

impl Backend {
    pub(crate) fn new(initial_root: PathBuf, client: Client) -> Self {
        let documents = Arc::new(SynchronizedDocuments::new(initial_root));
        Self {
            diagnostics: DiagnosticPublisher::new(client, Arc::clone(&documents)),
            documents,
        }
    }

    async fn configure_workspace(&self, params: &InitializeParams) -> Result<()> {
        let paths = InitializationPaths::from_params(params).map_err(Error::invalid_params)?;
        let Some(root) = paths.workspace_root else {
            return Ok(());
        };
        self.documents
            .set_workspace(&root, paths.standard_library_root.as_deref())
            .await
            .map_err(|error| Error::invalid_params(error.to_string()))
    }

    fn log_sync_error(operation: &str, error: impl std::fmt::Display) {
        tracing::warn!(operation, %error, "rejected document synchronization notification");
    }

    fn schedule_diagnostics(&self, document: SynchronizedDocument, generation: Option<u64>) {
        if let Some(generation) = generation {
            self.diagnostics.schedule(document, generation);
        }
    }

    async fn restore_current_diagnostics(&self, path: PathBuf, uri: Uri) {
        let Some(version) = self.documents.open_version(&path).await else {
            return;
        };
        let generation = self.diagnostics.invalidate(&path).await;
        self.schedule_diagnostics(SynchronizedDocument { path, uri, version }, generation);
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.configure_workspace(&params).await?;
        tracing::info!("language server initialized");
        Ok(capabilities::initialize_result())
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("language client acknowledged initialization");
    }

    async fn shutdown(&self) -> Result<()> {
        self.diagnostics.shutdown().await;
        self.documents.barrier().await;
        tracing::info!("language server shutdown requested");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let path = match file_uri_to_path(&uri) {
            Ok(path) => path,
            Err(error) => {
                Self::log_sync_error("didOpen", error);
                return;
            }
        };
        let generation = self.diagnostics.invalidate(&path).await;
        match self.documents.open(params).await {
            Ok(document) => self.schedule_diagnostics(document, generation),
            Err(error) => {
                Self::log_sync_error("didOpen", error);
                self.restore_current_diagnostics(path, uri).await;
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let path = match SynchronizedDocuments::validate_change(&params) {
            Ok(path) => path,
            Err(error) => {
                Self::log_sync_error("didChange", error);
                return;
            }
        };
        let generation = self.diagnostics.invalidate(&path).await;
        match self.documents.change(params).await {
            Ok(document) => self.schedule_diagnostics(document, generation),
            Err(error) => {
                Self::log_sync_error("didChange", error);
                self.restore_current_diagnostics(path, uri).await;
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Err(error) = self.documents.save(params).await {
            Self::log_sync_error("didSave", error);
        }
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let paths = params
            .changes
            .iter()
            .filter_map(|change| match file_uri_to_path(&change.uri) {
                Ok(path) => Some(path),
                Err(error) => {
                    Self::log_sync_error("didChangeWatchedFiles", error);
                    None
                }
            })
            .collect::<Vec<_>>();
        match self.documents.refresh_paths(paths).await {
            Ok(documents) => {
                for document in documents {
                    let generation = self.diagnostics.invalidate(&document.path).await;
                    self.schedule_diagnostics(document, generation);
                }
            }
            Err(error) => Self::log_sync_error("didChangeWatchedFiles", error),
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let path = match file_uri_to_path(&uri) {
            Ok(path) => path,
            Err(error) => {
                Self::log_sync_error("didClose", error);
                return;
            }
        };
        self.diagnostics.cancel(&path).await;
        match self.documents.close(params).await {
            Ok(()) => self.diagnostics.cancel_and_clear(&path, uri).await,
            Err(error) => Self::log_sync_error("didClose", error),
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let path =
            file_uri_to_path(&uri).map_err(|error| Error::invalid_params(error.to_string()))?;
        match self.documents.format_open(&path).await {
            Ok(Some(formatted)) => whole_document_edit(formatted).map(Some).map_err(|error| {
                tracing::warn!(path = %path.display(), %error, "cannot create formatting edit");
                Error::internal_error()
            }),
            Ok(None) => Ok(None),
            Err(error) => Err(Error::invalid_params(error.to_string())),
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.document_symbol_request(params).await
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        self.workspace_symbol_request(params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.hover_request(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.definition_request(params).await
    }

    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        self.type_definition_request(params).await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.document_highlight_request(params).await
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        self.selection_range_request(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.completion_request(params).await
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        self.completion_resolve_request(item).await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        self.signature_help_request(params).await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.references_request(params).await
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        self.prepare_rename_request(params).await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.rename_request(params).await
    }
}
