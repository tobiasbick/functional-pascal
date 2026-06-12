//! Path discovery for `fpas fmt` (directories, globs, projects).

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli_input::CliInput;
use fpas_project as project;
use glob::glob;

const SOURCE_FILE_EXTENSION: &str = "fpas";
const PROJECT_FILE_EXTENSION: &str = "fpasprj";
const WORKSPACE_FILE_EXTENSION: &str = "fpasworkspace";

/// Collects `.fpas` paths from explicit CLI arguments, or from discovery when `args` is empty.
pub(super) fn collect_format_paths(
    cwd: &Path,
    explicit_args: &[String],
    stderr: &mut dyn Write,
) -> Result<Vec<PathBuf>, i32> {
    if explicit_args.is_empty() {
        let input = crate::cli_input::discover_fmt_input(cwd).map_err(|message| {
            let _ = writeln!(stderr, "{message}");
            1
        })?;
        return collect_input_paths(&input, stderr);
    }

    let mut paths = Vec::new();
    for arg in explicit_args {
        match resolve_fmt_arg(arg, cwd, stderr) {
            Ok(mut resolved) => paths.append(&mut resolved),
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return Err(1);
            }
        }
    }

    Ok(dedupe_paths(paths))
}

fn collect_input_paths(input: &CliInput, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, i32> {
    match input {
        CliInput::SourceFile(path) => Ok(vec![path.clone()]),
        CliInput::ProjectFile(path) => collect_project_paths(path, stderr),
        CliInput::WorkspaceFile(path) => collect_workspace_paths(path, stderr),
    }
}

fn collect_project_paths(path: &Path, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, i32> {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return Err(1);
        }
    };

    for warning in &loaded.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }

    Ok(loaded.source_files)
}

fn collect_workspace_paths(path: &Path, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, i32> {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return Err(1);
        }
    };

    let mut paths = Vec::new();
    for member in &workspace.member_projects {
        paths.extend(collect_project_paths(member, stderr)?);
    }

    Ok(paths)
}

fn resolve_fmt_arg(arg: &str, cwd: &Path, stderr: &mut dyn Write) -> Result<Vec<PathBuf>, String> {
    if contains_glob_metacharacters(arg) {
        return expand_glob(arg, cwd);
    }

    let path = normalize_input_path(arg, cwd);
    if path.is_dir() {
        let files = collect_fpas_files_in_dir(&path);
        if files.is_empty() {
            return Err(format!(
                "No `.fpas` files found under `{}`.",
                path.display()
            ));
        }
        return Ok(files);
    }

    if has_extension(&path, SOURCE_FILE_EXTENSION) {
        return Ok(vec![path]);
    }
    if has_extension(&path, PROJECT_FILE_EXTENSION) {
        return collect_input_paths(&CliInput::ProjectFile(path), stderr)
            .map_err(|_| "failed to load project".to_string());
    }
    if has_extension(&path, WORKSPACE_FILE_EXTENSION) {
        return collect_input_paths(&CliInput::WorkspaceFile(path), stderr)
            .map_err(|_| "failed to load workspace".to_string());
    }

    Err(format!(
        "Unsupported input `{arg}`. Expected a `.fpas` file, directory, `.fpasprj`, or `.fpasworkspace`."
    ))
}

fn expand_glob(pattern: &str, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let pattern_path = normalize_input_path(pattern, cwd);
    let pattern_text = pattern_path.to_string_lossy();
    let mut matches = Vec::new();

    for entry in glob(&pattern_text).map_err(|error| {
        format!(
            "Invalid glob pattern `{pattern}`.\n  help: Use a pattern such as `src/**/*.fpas`.\n  details: {error}"
        )
    })? {
        let entry = entry.map_err(|error| {
            format!("Error while evaluating glob pattern `{pattern}`.\n  details: {error}")
        })?;
        if entry.is_file() && has_extension(&entry, SOURCE_FILE_EXTENSION) {
            matches.push(entry);
        }
    }

    if matches.is_empty() {
        return Err(format!(
            "Glob pattern `{pattern}` matched no `.fpas` files.\n  help: Check the path relative to `{}`.",
            cwd.display()
        ));
    }

    Ok(dedupe_paths(matches))
}

fn collect_fpas_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_fpas_files(dir, &mut files);
    files.sort();
    files
}

fn walk_fpas_files(dir: &Path, out: &mut Vec<PathBuf>) {
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
            walk_fpas_files(&path, out);
            continue;
        }
        if has_extension(&path, SOURCE_FILE_EXTENSION) {
            out.push(path);
        }
    }
}

fn contains_glob_metacharacters(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

fn normalize_input_path(input: &str, cwd: &Path) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = paths;
    deduped.sort();
    deduped.dedup();
    deduped
}
