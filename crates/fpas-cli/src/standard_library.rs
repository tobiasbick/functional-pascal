//! Resolution of the implementation-owned source standard library.

use std::path::{Path, PathBuf};

/// Resolves an explicit override or the `lib` directory beside the executable.
pub(crate) fn resolve_standard_library(
    override_path: Option<&Path>,
) -> Result<Option<fpas_project::StandardLibrary>, String> {
    let root = match override_path {
        Some(path) => path.to_path_buf(),
        None => match default_standard_library_root()? {
            Some(root) => root,
            None => return Ok(None),
        },
    };

    fpas_project::load_standard_library(&root).map(Some)
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
