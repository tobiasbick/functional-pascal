#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Native stdio entry point for the Functional Pascal language server.

use std::path::PathBuf;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => {}
        [arg] if arg == "--help" || arg == "-h" => {
            println!(
                "Functional Pascal language server\n\nUsage: fpas-lsp\n\n\
                 Reads LSP messages from stdin, writes protocol replies to stdout,\n\
                 and logs to stderr. Set initializationOptions.standardLibraryUri\n\
                 to the toolchain's source standard-library directory.\n\n\
                 Options:\n  -h, --help     Print this help\n  -V, --version  Print version"
            );
            return;
        }
        [arg] if arg == "--version" || arg == "-V" => {
            println!("fpas-lsp {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        _ => {
            eprintln!(
                "Usage: fpas-lsp\n  help: `fpas-lsp --help` describes the standard-I/O server."
            );
            std::process::exit(1);
        }
    }

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
    if let Err(error) = fpas_lsp::serve_stdio_blocking(initial_root) {
        tracing::error!(%error, "language server failed");
        std::process::exit(1);
    }
    tracing::info!("Functional Pascal language server stopped");
}
