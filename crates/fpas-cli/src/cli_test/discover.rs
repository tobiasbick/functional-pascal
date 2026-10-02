//! Test file discovery for `fpas test`.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::path::{Path, PathBuf};

use crate::CliInput;
use crate::cli_output::CliFailure;
use crate::cli_paths::{collect_files_in_dir, normalize_path};
use fpas_diagnostics::codes::{
    CLI_ARGUMENTS_INVALID, CLI_INPUT_UNSUPPORTED, PROJECT_DIRECTORY_READ_FAILED,
};
use fpas_project as project;

/// Returns sorted paths to `*_test.fpas` files for the CLI input.
pub(super) fn discover_test_files(
    input: &CliInput,
    cwd: &Path,
) -> Result<Vec<PathBuf>, CliFailure> {
    match input {
        CliInput::SourceFile(path) => discover_from_path(path, cwd),
        CliInput::ProjectFile(path) => discover_from_project(path),
        CliInput::WorkspaceFile(path) => discover_from_workspace(path),
        CliInput::CompiledProgramFile(path) => Err(CliFailure::new(
            CLI_INPUT_UNSUPPORTED,
            format!(
                "Cannot discover tests in compiled program `{}`.",
                path.display()
            ),
        )
        .with_help("Pass a test source, project, workspace, or directory.")),
    }
}

fn discover_from_path(path: &Path, cwd: &Path) -> Result<Vec<PathBuf>, CliFailure> {
    let resolved = normalize_path(path, cwd);
    if resolved.is_dir() {
        return collect_files_in_dir(&resolved, project::is_test_source_file)
            .map_err(|message| CliFailure::from_message(PROJECT_DIRECTORY_READ_FAILED, &message));
    }

    if project::is_test_source_file(&resolved) {
        return Ok(vec![resolved]);
    }

    Err(CliFailure::new(
        CLI_INPUT_UNSUPPORTED,
        format!("`{}` is not a test file.", resolved.display()),
    )
    .with_help(
        "Test files must be named `*_test.fpas`, or pass a directory or `.fpasprj` project.",
    ))
}

fn discover_from_project(project_path: &Path) -> Result<Vec<PathBuf>, CliFailure> {
    let loaded = project::load_project(project_path)?;
    Ok(filter_test_files(loaded.source_files))
}

fn discover_from_workspace(workspace_path: &Path) -> Result<Vec<PathBuf>, CliFailure> {
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

/// Selects exact discovered files, preserving discovery order and project context.
pub(super) fn select_test_paths(
    paths: Vec<PathBuf>,
    selected: &[PathBuf],
    cwd: &Path,
) -> Result<Vec<PathBuf>, CliFailure> {
    let mut identities = std::collections::HashSet::new();
    for selected_path in selected {
        let path = normalize_path(selected_path, cwd);
        let canonical = path.canonicalize().map_err(|error| {
            CliFailure::new(
                CLI_ARGUMENTS_INVALID,
                format!("Cannot select test `{}`: {error}.", path.display()),
            )
            .with_help("Use a path from `fpas test --list`.")
        })?;
        identities.insert(canonical);
    }
    let mut output = Vec::new();
    for path in paths {
        if let Ok(identity) = path.canonicalize()
            && identities.remove(&identity)
        {
            output.push(path);
        }
    }
    if let Some(missing) = identities.iter().next() {
        return Err(CliFailure::new(
            CLI_ARGUMENTS_INVALID,
            format!(
                "Selected test `{}` is not in the discovered test set.",
                missing.display()
            ),
        )
        .with_help("Use the owning project and a path from `fpas test --list`."));
    }
    Ok(output)
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
