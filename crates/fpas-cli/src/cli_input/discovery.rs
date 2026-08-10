//! Project and workspace discovery for CLI commands.

use std::fs;
use std::path::{Path, PathBuf};

use fpas_project::discover_workspace_file;

use crate::cli_paths::{
    COMPILED_PROGRAM_FILE_EXTENSION, PROJECT_FILE_EXTENSION, SOURCE_FILE_EXTENSION,
    WORKSPACE_FILE_EXTENSION, has_extension, normalize_input_path,
};

use super::mode::CliMode;
use super::types::CliInput;

pub(super) fn resolve_explicit_input(
    input: &str,
    cwd: &Path,
    mode: CliMode,
) -> Result<CliInput, String> {
    let path = normalize_input_path(input, cwd);
    if path.is_dir() {
        if mode == CliMode::Build {
            return Err(format!(
                "Unsupported build input `{}`. Expected a `.fpasprj` or `.fpasworkspace` file.",
                path.display()
            ));
        }
        return Ok(CliInput::SourceFile(path));
    }
    if has_extension(&path, SOURCE_FILE_EXTENSION) {
        if mode == CliMode::Build {
            return Err(format!(
                "Unsupported build input `{}`. Expected a `.fpasprj` or `.fpasworkspace` file.",
                path.display()
            ));
        }
        if mode == CliMode::Test {
            crate::cli_test::validate_explicit_test_file(&path)?;
        }
        return Ok(CliInput::SourceFile(path));
    }
    if has_extension(&path, PROJECT_FILE_EXTENSION) {
        return Ok(CliInput::ProjectFile(path));
    }
    if matches!(
        mode,
        CliMode::Build
            | CliMode::Run
            | CliMode::Debug
            | CliMode::Check
            | CliMode::Fmt
            | CliMode::Test
    ) && has_extension(&path, WORKSPACE_FILE_EXTENSION)
    {
        return Ok(CliInput::WorkspaceFile(path));
    }
    if matches!(mode, CliMode::Run | CliMode::Debug)
        && has_extension(&path, COMPILED_PROGRAM_FILE_EXTENSION)
    {
        return Ok(CliInput::CompiledProgramFile(path));
    }

    let expected = match mode {
        CliMode::Build => "a `.fpasprj` or `.fpasworkspace` file",
        CliMode::Run => "a `.fpas`, `.fpasprj`, `.fpasworkspace`, or `.fpascp` file",
        CliMode::Debug => "a `.fpas`, `.fpasprj`, `.fpasworkspace`, or `.fpascp` file",
        CliMode::Check => "a `.fpas` file, directory, `.fpasprj`, or `.fpasworkspace` file",
        CliMode::Fmt => "a `.fpas`, `.fpasprj`, or `.fpasworkspace` file",
        CliMode::Test => "a `.fpas` file, directory, `.fpasprj`, or `.fpasworkspace` file",
    };
    Err(format!(
        "Unsupported input `{}`. Expected {expected}.",
        path.display()
    ))
}

pub(super) fn discover_input(cwd: &Path, mode: CliMode) -> Result<CliInput, String> {
    match mode {
        CliMode::Build | CliMode::Check | CliMode::Fmt | CliMode::Test => discover_check_input(cwd),
        CliMode::Run | CliMode::Debug => discover_run_input(cwd),
    }
}

fn discover_run_input(cwd: &Path) -> Result<CliInput, String> {
    if let Some(workspace_path) = discover_workspace_file(cwd)? {
        return Ok(CliInput::WorkspaceFile(workspace_path));
    }

    discover_project_file(cwd)
}

/// Discovers workspace or project input for `fpas check`, `fpas fmt`, and `fpas test` when no path is given.
pub(crate) fn discover_check_input(cwd: &Path) -> Result<CliInput, String> {
    if let Some(workspace_path) = discover_workspace_file(cwd)? {
        return Ok(CliInput::WorkspaceFile(workspace_path));
    }

    discover_project_file(cwd)
}

fn discover_project_file(cwd: &Path) -> Result<CliInput, String> {
    let read_dir = fs::read_dir(cwd)
        .map_err(|e| format!("Error reading current directory `{}`: {e}", cwd.display()))?;

    let mut candidates = Vec::<PathBuf>::new();
    for entry in read_dir {
        let entry = entry.map_err(|e| {
            format!(
                "Error reading entries from current directory `{}`: {e}",
                cwd.display()
            )
        })?;
        let path = entry.path();
        if path.is_file() && has_extension(&path, PROJECT_FILE_EXTENSION) {
            candidates.push(path);
        }
    }

    candidates.sort();

    match candidates.len() {
        0 => Err(format!(
            "No `.fpasprj` file found in current directory `{}`.\n  help: Pass a `.fpas`, `.fpasprj`, or `.fpasworkspace` path explicitly.",
            cwd.display()
        )),
        1 => Ok(CliInput::ProjectFile(candidates.remove(0))),
        _ => {
            let entries = candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "Found multiple `.fpasprj` files in current directory `{}`: {entries}.\n  help: Pass the desired `.fpasprj` file path explicitly.",
                cwd.display()
            ))
        }
    }
}
