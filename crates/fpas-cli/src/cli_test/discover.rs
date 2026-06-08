//! Test file discovery for `fpas test`.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::fs;
use std::path::{Path, PathBuf};

use crate::CliInput;
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

    if is_test_file_name(&resolved) {
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
        .filter(|path| is_test_file_name(path))
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
            collect_test_files_recursive_inner(&path, out);
        } else if is_test_file_name(&path) {
            out.push(path);
        }
    }
}

/// Returns true when `path` ends with `_test.fpas` (case-insensitive).
pub(super) fn is_test_file_name(path: &Path) -> bool {
    project::is_test_source_file(path)
}

fn normalize_path(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
