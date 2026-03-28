use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_FILE_EXTENSION: &str = "fpas";

pub(super) fn resolve_source_files(
    entries: &[String],
    root_dir: &Path,
) -> Result<(Vec<PathBuf>, Vec<String>), String> {
    let mut files = Vec::<PathBuf>::new();
    let mut warnings = Vec::<String>::new();
    let mut seen = HashSet::<PathBuf>::new();

    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            return Err(
                "A `sources.include` entry is empty.\n  help: Remove empty entries or provide a file path/pattern."
                    .to_string(),
            );
        }

        if is_glob_pattern(entry) {
            let pattern_path = resolve_path(entry, root_dir);
            let pattern_text = pattern_path.to_string_lossy().replace('\\', "/");
            let mut matches = Vec::<PathBuf>::new();
            for matched in glob(&pattern_text).map_err(|e| {
                format!(
                    "Invalid glob pattern `{entry}`.\n  help: Use a valid glob such as `src/**/*.fpas`.\n  details: {e}"
                )
            })? {
                let matched = matched.map_err(|e| {
                    format!(
                        "Error while evaluating glob pattern `{entry}`.\n  details: {e}"
                    )
                })?;
                if matched.is_file() {
                    matches.push(matched);
                }
            }

            if matches.is_empty() {
                return Err(format!(
                    "Include pattern `{entry}` matched no files.\n  help: Check the path or pattern relative to the project directory."
                ));
            }

            matches.sort();
            for matched in matches {
                if !has_source_extension(&matched) {
                    return Err(format!(
                        "Include pattern `{entry}` matched a non-source file `{}`.\n  help: Restrict the pattern to `.fpas` files (for example `src/**/*.fpas`).",
                        matched.to_string_lossy()
                    ));
                }
                insert_unique_source_file(matched, &mut files, &mut seen, &mut warnings);
            }
            continue;
        }

        let explicit_path = resolve_explicit_file_path("sources.include", entry, root_dir)?;
        validate_source_extension(&explicit_path, "sources.include")?;
        insert_unique_source_file(explicit_path, &mut files, &mut seen, &mut warnings);
    }

    Ok((files, warnings))
}

fn insert_unique_source_file(
    path: PathBuf,
    files: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
    warnings: &mut Vec<String>,
) {
    let key = canonical_or_original(path.as_path());
    if seen.insert(key) {
        files.push(path);
        return;
    }

    warnings.push(format!(
        "Duplicate source file `{}` was ignored.",
        path.to_string_lossy()
    ));
}

pub(super) fn resolve_explicit_file_path(
    field_name: &str,
    value: &str,
    root_dir: &Path,
) -> Result<PathBuf, String> {
    validate_non_empty(field_name, value)?;
    let path = resolve_path(value, root_dir);
    if !path.exists() {
        return Err(format!(
            "`{field_name}` path does not exist: `{}`.\n  help: Use an existing file path.",
            path.to_string_lossy()
        ));
    }
    if !path.is_file() {
        return Err(format!(
            "`{field_name}` must point to a file: `{}`.\n  help: Use a file path instead of a directory.",
            path.to_string_lossy()
        ));
    }

    Ok(path)
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "`{field_name}` must be a non-empty string.\n  help: Provide a value such as `\"my-app\"`."
        ));
    }

    Ok(())
}

fn resolve_path(value: &str, root_dir: &Path) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root_dir.join(path)
    }
}

fn is_glob_pattern(value: &str) -> bool {
    value.chars().any(|c| matches!(c, '*' | '?' | '[' | ']'))
}

fn has_source_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(SOURCE_FILE_EXTENSION))
}

pub(super) fn validate_source_extension(path: &Path, field_name: &str) -> Result<(), String> {
    if has_source_extension(path) {
        return Ok(());
    }

    Err(format!(
        "`{field_name}` must reference a `.fpas` file: `{}`.\n  help: Use a `.fpas` source file path.",
        path.to_string_lossy()
    ))
}

pub(super) fn same_file(left: &Path, right: &Path) -> bool {
    canonical_or_original(left) == canonical_or_original(right)
}

fn canonical_or_original(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
