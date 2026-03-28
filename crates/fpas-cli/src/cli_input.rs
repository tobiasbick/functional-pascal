use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_FILE_EXTENSION: &str = "fpas";
const PROJECT_FILE_EXTENSION: &str = "fpasprj";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CliInput {
    SourceFile(PathBuf),
    ProjectFile(PathBuf),
}

pub(crate) fn resolve_cli_input(args: &[String], cwd: &Path) -> Result<CliInput, String> {
    match args {
        [] => discover_project_file(cwd),
        [input] => resolve_explicit_input(input, cwd),
        _ => Err("Usage: fpas [<file.fpas | file.fpasprj>]".to_string()),
    }
}

fn resolve_explicit_input(input: &str, cwd: &Path) -> Result<CliInput, String> {
    let path = normalize_input_path(input, cwd);
    if has_extension(&path, SOURCE_FILE_EXTENSION) {
        return Ok(CliInput::SourceFile(path));
    }
    if has_extension(&path, PROJECT_FILE_EXTENSION) {
        return Ok(CliInput::ProjectFile(path));
    }

    Err(format!(
        "Unsupported input `{}`. Expected a `.fpas` or `.fpasprj` file.",
        path.display()
    ))
}

fn normalize_input_path(input: &str, cwd: &Path) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
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
            "No `.fpasprj` file found in current directory `{}`.\n  help: Pass a `.fpas` or `.fpasprj` path explicitly.",
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

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}
