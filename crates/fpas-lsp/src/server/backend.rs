//! `tower-lsp-server` backend for Phase 4 lifecycle and document synchronization.

use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
};

use crate::capabilities;
use crate::convert::file_uri_to_path;
use crate::documents::SynchronizedDocuments;

/// Functional Pascal LSP backend with full-text synchronized documents.
pub struct Backend {
    documents: Arc<SynchronizedDocuments>,
}

impl Backend {
    pub(crate) fn new(initial_root: PathBuf) -> Self {
        Self {
            documents: Arc::new(SynchronizedDocuments::new(initial_root)),
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
        self.documents.barrier().await;
        tracing::info!("language server shutdown requested");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Err(error) = self.documents.open(params).await {
            Self::log_sync_error("didOpen", error);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(error) = self.documents.change(params).await {
            Self::log_sync_error("didChange", error);
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Err(error) = self.documents.save(params).await {
            Self::log_sync_error("didSave", error);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        if let Err(error) = self.documents.close(params).await {
            Self::log_sync_error("didClose", error);
        }
    }
}
