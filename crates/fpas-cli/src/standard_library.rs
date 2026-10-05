//! Resolution of the implementation-owned source standard library.

use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::PROJECT_STANDARD_LIBRARY_INVALID;

use crate::cli_output::CliFailure;

/// Resolves an explicit override or the `lib` directory beside the executable.
pub(crate) fn resolve_standard_library(
    override_path: Option<&Path>,
) -> Result<Option<fpas_project::StandardLibrary>, CliFailure> {
    let root = match override_path {
        Some(path) => path.to_path_buf(),
        None => match default_standard_library_root().map_err(|message| {
            CliFailure::new(PROJECT_STANDARD_LIBRARY_INVALID, message)
                .with_help("Pass `--std-lib <directory>` containing `stdlib.fpasprj`.")
        })? {
            Some(root) => root,
            None => return Ok(None),
        },
    };

    Ok(Some(fpas_project::load_standard_library(&root)?))
}

/// Returns the source standard library installed beside the running toolchain.
pub(crate) fn default_standard_library_root() -> Result<Option<PathBuf>, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Cannot locate the running `fpas` executable: {error}"))?;
    let Some(executable_dir) = executable.parent() else {
        return Ok(None);
    };
    let adjacent = executable_dir.join("lib");
    if adjacent.join("stdlib.fpasprj").is_file() {
        return Ok(Some(adjacent));
    }

    // Cargo test binaries live in `target/<profile>/deps`; the CLI binary and copied
    // development library live one directory above.
    let development_library = executable_dir
        .parent()
        .map(|profile_dir| profile_dir.join("lib"));
    Ok(development_library.filter(|root| root.join("stdlib.fpasprj").is_file()))
}
