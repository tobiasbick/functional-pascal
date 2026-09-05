use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf, absolute};

use crate::source::validate_non_empty;
use crate::{PathGlobError, expand_path_glob};

const SOURCE_FILE_EXTENSION: &str = "fpas";

/// Resolves `sources.include` and applies optional `sources.exclude` patterns.
///
/// Documentation: `docs/pascal/program-structure/projects.md`
pub(super) fn resolve_source_files(
    include: &[String],
    exclude: &[String],
    root_dir: &Path,
) -> Result<(Vec<PathBuf>, Vec<String>), String> {
    let (mut files, warnings) = expand_include_entries(include, root_dir)?;
    if !exclude.is_empty() {
        let excluded = expand_exclude_entries(exclude, root_dir)?;
        files.retain(|path| {
            let key = canonical_or_original(path.as_path());
            !excluded
                .iter()
                .any(|excluded_path| canonical_or_original(excluded_path.as_path()) == key)
        });
    }

    Ok((files, warnings))
}

fn expand_include_entries(
    entries: &[String],
    root_dir: &Path,
) -> Result<(Vec<PathBuf>, Vec<String>), String> {
    let mut files = Vec::<PathBuf>::new();
    let mut warnings = Vec::<String>::new();
    let mut seen = HashSet::<PathBuf>::new();

    for entry in entries {
        for matched in expand_source_pattern("sources.include", entry, root_dir, true)? {
            validate_source_extension(&matched, "sources.include")?;
            insert_unique_source_file(matched, &mut files, &mut seen, &mut warnings);
        }
    }

    Ok((files, warnings))
}

fn expand_exclude_entries(entries: &[String], root_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut excluded = Vec::<PathBuf>::new();
    let mut seen = HashSet::<PathBuf>::new();

    for entry in entries {
        for matched in expand_source_pattern("sources.exclude", entry, root_dir, false)? {
            let key = canonical_or_original(matched.as_path());
            if seen.insert(key) {
                excluded.push(matched);
            }
        }
    }

    Ok(excluded)
}

fn expand_source_pattern(
    field_name: &str,
    entry: &str,
    root_dir: &Path,
    require_glob_match: bool,
) -> Result<Vec<PathBuf>, String> {
    let entry = entry.trim();
    if entry.is_empty() {
        return Err(format!(
            "A `{field_name}` entry is empty.\n  help: Remove empty entries or provide a file path/pattern."
        ));
    }

    let resolved_path = resolve_path(entry, root_dir);
    if resolved_path.is_file() {
        return Ok(vec![resolved_path]);
    }

    if is_glob_pattern(entry) {
        let normalized_entry = entry.replace('\\', "/");
        let mut matches = expand_path_glob(root_dir, &normalized_entry).map_err(|error| {
            match error {
                PathGlobError::InvalidPattern(error) => format!(
                    "Invalid glob pattern `{entry}` in `{field_name}`.\n  help: Use a valid glob such as `src/**/*.fpas`.\n  details: {error}"
                ),
                error => format!(
                    "Error while evaluating glob pattern `{entry}` in `{field_name}`.\n  details: {error}"
                ),
            }
        })?;
        matches.retain(|matched| matched.is_file());

        if require_glob_match && matches.is_empty() {
            return Err(format!(
                "Pattern `{entry}` in `{field_name}` matched no files.\n  help: Check the path or pattern relative to the project directory."
            ));
        }

        return Ok(matches);
    }

    let explicit_path = resolve_explicit_file_path(field_name, entry, root_dir)?;
    Ok(vec![explicit_path])
}

const PROJECT_FILE_EXTENSION: &str = "fpasprj";

/// Absolute manifest path used at public project-loading boundaries.
pub(crate) fn absolute_project_path(path: &Path) -> Result<PathBuf, String> {
    absolute(path).map_err(|error| {
        format!(
            "Cannot resolve absolute project path for `{}`: {error}",
            path.display()
        )
    })
}

/// Canonical path used for project dependency graphs and deduplication.
pub(crate) fn canonical_project_path(path: &Path) -> PathBuf {
    canonical_or_original(path)
}

/// Appends `incoming` to `target`, ignoring duplicate source files with a warning.
pub(super) fn merge_source_files(
    target: &mut Vec<PathBuf>,
    incoming: Vec<PathBuf>,
    warnings: &mut Vec<String>,
) {
    let mut seen = target
        .iter()
        .map(|path| canonical_or_original(path.as_path()))
        .collect::<HashSet<_>>();

    for path in incoming {
        insert_unique_source_file(path, target, &mut seen, warnings);
    }
}

/// Resolves a `dependencies.projects` entry to an existing `.fpasprj` file.
pub(super) fn resolve_project_dependency_path(
    value: &str,
    root_dir: &Path,
) -> Result<PathBuf, String> {
    let path = resolve_explicit_file_path("dependencies.projects", value, root_dir)?;
    validate_project_file_extension(&path, "dependencies.projects")?;
    Ok(path)
}

fn validate_project_file_extension(path: &Path, field_name: &str) -> Result<(), String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(PROJECT_FILE_EXTENSION))
    {
        return Ok(());
    }

    Err(format!(
        "`{field_name}` must reference a `.fpasprj` file: `{}`.\n  help: Point each dependency at a library project manifest.",
        path.to_string_lossy()
    ))
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
        "Duplicate source file `{}` was ignored; the first occurrence was retained.",
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

pub(super) fn resolve_path(value: &str, root_dir: &Path) -> PathBuf {
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

pub(super) fn canonical_source_path(path: &Path) -> PathBuf {
    canonical_or_original(path)
}

/// Compares paths directly before resolving filesystem aliases.
pub(super) fn same_file(left: &Path, right: &Path) -> bool {
    left == right || canonical_or_original(left) == canonical_or_original(right)
}

fn canonical_or_original(path: &Path) -> PathBuf {
    fs::canonicalize(path)
        .or_else(|_| absolute(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::expand_source_pattern;
    use super::{canonical_or_original, same_file};
    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;
    use std::path::Path;

    #[test]
    fn canonical_or_original_falls_back_to_absolute_path_for_missing_entries() {
        let fallback = canonical_or_original(Path::new("missing-path-for-tests/example.fpas"));

        assert!(fallback.is_absolute());
    }

    #[test]
    fn same_file_accepts_identical_existing_and_missing_paths() {
        for path in ["Cargo.toml", "missing-path-for-tests/example.fpas"] {
            assert!(same_file(Path::new(path), Path::new(path)));
        }
    }

    #[test]
    fn same_file_resolves_relative_absolute_and_parent_aliases() {
        let relative = Path::new("Cargo.toml");
        let absolute = std::env::current_dir().unwrap().join(relative);
        assert!(same_file(relative, &absolute));
        assert!(same_file(relative, Path::new("src/../Cargo.toml")));
    }

    #[test]
    fn same_file_preserves_missing_path_fallback_without_merging_distinct_paths() {
        let missing = Path::new("missing-path-for-tests/example.fpas");
        assert!(same_file(
            missing,
            &std::env::current_dir().unwrap().join(missing)
        ));
        assert!(!same_file(
            missing,
            Path::new("missing-path-for-tests/other.fpas")
        ));
        assert!(!same_file(Path::new("Cargo.toml"), Path::new("src/lib.rs")));
    }

    #[cfg(windows)]
    #[test]
    fn same_file_resolves_case_aliases_on_windows() {
        assert!(same_file(Path::new("Cargo.toml"), Path::new("CARGO.TOML")));
    }

    #[cfg(unix)]
    #[test]
    fn source_glob_matches_below_non_utf8_project_directory() {
        let mut directory_name =
            format!("fpas-project-non-utf8-{}-", std::process::id()).into_bytes();
        directory_name.push(0xff);
        let root = std::env::temp_dir().join(std::ffi::OsString::from_vec(directory_name));
        let source = root.join("src/nested.fpas");
        fs::create_dir_all(source.parent().expect("source must have a parent"))
            .expect("source directory must be created");
        fs::write(&source, "program Nested; begin end.").expect("source fixture must be written");

        let result = expand_source_pattern("sources.include", "src/**/*.fpas", &root, true);
        fs::remove_dir_all(&root).expect("fixture must be removed");

        assert_eq!(result, Ok(vec![source]));
    }
}
