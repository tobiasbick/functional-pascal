//! `tower-lsp-server` backend for lifecycle, documents, diagnostics, and formatting.

use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::{
    CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentFormattingParams,
    DocumentSymbolParams, DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse,
    Hover, HoverParams, InitializeParams, InitializeResult, InitializedParams, TextEdit, Uri,
};
use tower_lsp_server::{Client, LanguageServer};

use crate::capabilities;
use crate::convert::file_uri_to_path;
use crate::diagnostics::DiagnosticPublisher;
use crate::documents::{SynchronizedDocument, SynchronizedDocuments};
use crate::formatting::whole_document_edit;

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

    async fn configure_workspace(&self, params: &InitializeParams) {
        let root_uri = initialization_root_uri(params);
        let Some(root_uri) = root_uri else {
            return;
        };
        match file_uri_to_path(root_uri) {
            Ok(root) => self.documents.set_workspace_root(&root).await,
            Err(error) => tracing::warn!(%error, "ignoring unsupported workspace root"),
        }
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

#[expect(
    deprecated,
    reason = "LSP 3.17 permits rootUri when workspaceFolders is unavailable"
)]
fn initialization_root_uri(params: &InitializeParams) -> Option<&tower_lsp_server::ls_types::Uri> {
    params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .map(|folder| &folder.uri)
        .or(params.root_uri.as_ref())
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.configure_workspace(&params).await;
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

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.hover_request(params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.definition_request(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.completion_request(params).await
    }
}
