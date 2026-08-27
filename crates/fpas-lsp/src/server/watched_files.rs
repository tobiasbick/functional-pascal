//! Dynamic registration for source and project file changes.

use tower_lsp_server::Client;
use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::ls_types::{
    DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, Registration,
};

const WATCHED_FILE_GLOBS: [&str; 3] = ["**/*.fpas", "**/*.fpasprj", "**/*.fpasworkspace"];

pub(super) async fn register(client: &Client) -> Result<()> {
    let options = DidChangeWatchedFilesRegistrationOptions {
        watchers: WATCHED_FILE_GLOBS
            .iter()
            .map(|pattern| FileSystemWatcher {
                glob_pattern: GlobPattern::String((*pattern).to_owned()),
                kind: None,
            })
            .collect(),
    };
    let register_options = serde_json::to_value(options).map_err(|error| {
        tracing::error!(%error, "cannot serialize watched-file registration");
        Error::internal_error()
    })?;
    client
        .register_capability(vec![Registration {
            id: "fpas-watched-files".to_owned(),
            method: "workspace/didChangeWatchedFiles".to_owned(),
            register_options: Some(register_options),
        }])
        .await
}
