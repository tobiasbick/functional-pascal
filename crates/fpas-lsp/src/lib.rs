//! Standard-LSP transport for the Functional Pascal language service.
//!
//! This crate owns protocol lifecycle, URI and UTF-16 conversion, and full-text document
//! synchronization plus diagnostic publication and formatting edits. Language behavior remains in
//! `fpas-language-service` and its compiler crates.

#![deny(missing_docs)]

mod capabilities;
pub mod convert;
mod diagnostics;
mod dispatch;
mod documents;
mod formatting;
mod intellisense;
mod intellisense_requests;
mod navigation;
mod navigation_requests;
mod semantic_tools;
mod semantic_tools_requests;
mod server;

use std::path::PathBuf;

use tower_lsp_server::{LspService, Server};

pub use dispatch::OrderedLspService;
pub use server::Backend;

/// Creates an in-memory LSP service and its client socket for protocol tests or custom transports.
#[must_use]
pub fn create_service(
    initial_root: PathBuf,
) -> (OrderedLspService, tower_lsp_server::ClientSocket) {
    let (service, socket) = LspService::new(move |client| Backend::new(initial_root, client));
    (OrderedLspService::new(service), socket)
}

/// Serves Functional Pascal LSP messages over standard input and output until the client exits.
pub async fn serve_stdio(initial_root: PathBuf) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = create_service(initial_root);
    Server::new(stdin, stdout, socket).serve(service).await;
}
