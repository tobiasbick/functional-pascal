//! Native stdio entry point for the Functional Pascal language server.

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    let initial_root = std::env::current_dir().unwrap_or_else(|error| {
        tracing::warn!(%error, "cannot read the process working directory; using a relative root");
        PathBuf::from(".")
    });
    tracing::info!("Functional Pascal language server starting");
    fpas_lsp::serve_stdio(initial_root).await;
    tracing::info!("Functional Pascal language server stopped");
}
