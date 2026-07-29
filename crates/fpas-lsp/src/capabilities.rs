//! Capabilities implemented by the Phase 4 transport.

use tower_lsp_server::ls_types::{
    InitializeResult, PositionEncodingKind, SaveOptions, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions,
};

pub(crate) fn initialize_result() -> InitializeResult {
    InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: Some(PositionEncodingKind::UTF16),
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(false),
                    })),
                },
            )),
            ..ServerCapabilities::default()
        },
        server_info: Some(ServerInfo {
            name: "Functional Pascal Language Server".to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        }),
        offset_encoding: None,
    }
}
