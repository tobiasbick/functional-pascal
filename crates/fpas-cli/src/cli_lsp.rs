//! Language-server entry through the selected `fpas` toolchain.

use std::io::Write;
use std::path::Path;

pub(crate) fn run_language_server(cwd: &Path, stderr: &mut dyn Write) -> i32 {
    if let Err(error) = fpas_lsp::serve_stdio_blocking(cwd.to_path_buf()) {
        let _ = writeln!(
            stderr,
            "Functional Pascal language server failed: {error}\n  help: Run `fpas env --json` to verify the installed toolchain."
        );
        return 1;
    }
    0
}
