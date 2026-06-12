//! Recursive `.fpas` discovery for integration tests.

use std::path::{Path, PathBuf};

/// Repository roots scanned by round-trip and fuzz-light tests (relative to workspace root).
pub const REPO_SOURCE_ROOTS: &[&str] = &["examples", "tests", "apps"];

/// Walks `dir` recursively and calls `visit` for each `.fpas` file (skips `target/`).
pub fn walk_fpas_files(dir: &Path, visit: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
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
            walk_fpas_files(&path, visit);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "fpas") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", path.display());
        });
        visit(&path, &source);
    }
}

/// Returns every `.fpas` path under `roots`, sorted for deterministic sampling.
#[must_use]
pub fn collect_fpas_paths(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for root in roots {
        collect_fpas_paths_in_dir(root, &mut paths);
    }
    paths.sort();
    paths.dedup();
    paths
}

fn collect_fpas_paths_in_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
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
            collect_fpas_paths_in_dir(&path, out);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "fpas") {
            continue;
        }
        out.push(path);
    }
}

/// Workspace-relative path to a repository source root.
#[must_use]
pub fn repo_root(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}
