//! Test file discovery for `fpas test`.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::fs;
use std::path::{Path, PathBuf};

use crate::CliInput;
use crate::cli_paths::normalize_path;
use fpas_project as project;

/// Returns sorted paths to `*_test.fpas` files for the CLI input.
pub(super) fn discover_test_files(input: &CliInput, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    match input {
        CliInput::SourceFile(path) => discover_from_path(path, cwd),
        CliInput::ProjectFile(path) => discover_from_project(path),
        CliInput::WorkspaceFile(path) => discover_from_workspace(path),
    }
}

fn discover_from_path(path: &Path, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let resolved = normalize_path(path, cwd);
    if resolved.is_dir() {
        return Ok(collect_test_files_recursive(&resolved));
    }

    if project::is_test_source_file(&resolved) {
        return Ok(vec![resolved]);
    }

    Err(format!(
        "`{}` is not a test file.\n  help: Test files must be named `*_test.fpas`, or pass a directory or `.fpasprj` project.",
        resolved.display()
    ))
}

fn discover_from_project(project_path: &Path) -> Result<Vec<PathBuf>, String> {
    let loaded = project::load_project(project_path)?;
    Ok(filter_test_files(loaded.source_files))
}

fn discover_from_workspace(workspace_path: &Path) -> Result<Vec<PathBuf>, String> {
    let test_members = project::discover_test_projects_in_workspace(workspace_path)?;
    let mut paths = Vec::new();
    for member in test_members {
        let loaded = project::load_project(&member)?;
        paths.extend(filter_test_files(loaded.source_files));
    }
    paths.sort();
    paths.dedup_by(|a, b| a == b);
    Ok(paths)
}

fn filter_test_files(source_files: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = source_files
        .into_iter()
        .filter(|path| project::is_test_source_file(path))
        .collect();
    paths.sort();
    paths
}

fn collect_test_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_test_files_recursive_inner(dir, &mut paths);
    paths.sort();
    paths
}

fn collect_test_files_recursive_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("target"))
            {
                continue;
            }
            collect_test_files_recursive_inner(&path, out);
        } else if project::is_test_source_file(&path) {
            out.push(path);
        }
    }
}

/// Keeps paths whose file name or full path contains `pattern` (case-insensitive).
pub(super) fn filter_test_paths(paths: Vec<PathBuf>, pattern: &str) -> Vec<PathBuf> {
    let needle = pattern.trim();
    if needle.is_empty() {
        return paths;
    }
    let needle = needle.to_lowercase();
    paths
        .into_iter()
        .filter(|path| path_matches_filter(path, &needle))
        .collect()
}

fn path_matches_filter(path: &Path, needle: &str) -> bool {
    path.to_string_lossy().to_lowercase().contains(needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_test_paths_matches_basename_substring() {
        let paths = vec![
            PathBuf::from("alpha_test.fpas"),
            PathBuf::from("beta_test.fpas"),
        ];
        let filtered = filter_test_paths(paths, "alpha");
        assert_eq!(filtered, vec![PathBuf::from("alpha_test.fpas")]);
    }

    #[test]
    fn filter_test_paths_empty_pattern_keeps_all() {
        let paths = vec![PathBuf::from("one_test.fpas")];
        assert_eq!(filter_test_paths(paths.clone(), ""), paths);
    }
}
