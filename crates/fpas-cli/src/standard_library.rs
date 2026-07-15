//! Resolution of the implementation-owned source standard library.

use std::path::Path;

/// Resolves an explicit override or the `lib` directory beside the executable.
pub(crate) fn resolve_standard_library(
    override_path: Option<&Path>,
) -> Result<Option<fpas_project::StandardLibrary>, String> {
    let root = match override_path {
        Some(path) => path.to_path_buf(),
        None => {
            let Some(executable) = std::env::current_exe().ok() else {
                return Ok(None);
            };
            let Some(executable_dir) = executable.parent() else {
                return Ok(None);
            };
            let adjacent = executable_dir.join("lib");
            if adjacent.is_dir() {
                adjacent
            } else {
                // Cargo test binaries live in `target/<profile>/deps`; the CLI binary and
                // its copied library live one directory above during development.
                let Some(profile_dir) = executable_dir.parent() else {
                    return Ok(None);
                };
                let development_library = profile_dir.join("lib");
                if !development_library.is_dir() {
                    return Ok(None);
                }
                development_library
            }
        }
    };

    fpas_project::load_standard_library(&root).map(Some)
}
