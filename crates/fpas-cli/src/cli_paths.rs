//! Shared path and file-extension helpers for CLI argument resolution.

use std::path::{Path, PathBuf};

/// Source file extension (without dot).
pub(crate) const SOURCE_FILE_EXTENSION: &str = "fpas";
/// Project manifest extension (without dot).
pub(crate) const PROJECT_FILE_EXTENSION: &str = "fpasprj";
/// Workspace manifest extension (without dot).
pub(crate) const WORKSPACE_FILE_EXTENSION: &str = "fpasworkspace";

/// Returns true when `path` has the given extension (ASCII case-insensitive).
pub(crate) fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

/// Resolves a CLI string path against `cwd` when relative.
pub(crate) fn normalize_input_path(input: &str, cwd: &Path) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

/// Resolves a `Path` against `cwd` when relative.
pub(crate) fn normalize_path(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
