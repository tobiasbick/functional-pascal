//! Shared path and file-extension helpers for CLI argument resolution.

use std::fs;
use std::path::{Path, PathBuf};

/// Source file extension (without dot).
pub(crate) const SOURCE_FILE_EXTENSION: &str = "fpas";
/// Project manifest extension (without dot).
pub(crate) const PROJECT_FILE_EXTENSION: &str = "fpasprj";
/// Workspace manifest extension (without dot).
pub(crate) const WORKSPACE_FILE_EXTENSION: &str = "fpasworkspace";
/// Compiled program extension (without dot).
pub(crate) const COMPILED_PROGRAM_FILE_EXTENSION: &str = "fpascp";

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

/// Returns sorted `.fpas` source paths under `dir`, skipping `target` directories.
pub(crate) fn collect_fpas_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_fpas_files(dir, &mut files);
    files.sort();
    files
}

fn walk_fpas_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("target"))
            {
                continue;
            }
            walk_fpas_files(&path, out);
            continue;
        }
        if has_extension(&path, SOURCE_FILE_EXTENSION) {
            out.push(path);
        }
    }
}
