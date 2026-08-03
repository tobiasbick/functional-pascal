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

/// Returns sorted matching files under `dir`, skipping `target` directories and symbolic links.
pub(crate) fn collect_files_in_dir(
    dir: &Path,
    matches: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>, String> {
    collect_files_with_reader(dir, &matches, &mut read_directory)
}

/// Returns sorted `.fpas` source paths under `dir`, skipping `target` directories.
pub(crate) fn collect_fpas_files_in_dir(dir: &Path) -> Result<Vec<PathBuf>, String> {
    collect_files_in_dir(dir, |path| has_extension(path, SOURCE_FILE_EXTENSION))
}

#[derive(Clone, Copy)]
enum EntryKind {
    Directory,
    File,
    Other,
}

struct WalkEntry {
    path: PathBuf,
    kind: EntryKind,
}

enum EntryError {
    Read(std::io::Error),
    FileType(PathBuf, std::io::Error),
}

fn read_directory(dir: &Path) -> std::io::Result<Vec<Result<WalkEntry, EntryError>>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                entries.push(Err(EntryError::Read(error)));
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                entries.push(Err(EntryError::FileType(path, error)));
                continue;
            }
        };
        let kind = if file_type.is_dir() {
            EntryKind::Directory
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };
        entries.push(Ok(WalkEntry { path, kind }));
    }
    Ok(entries)
}

fn collect_files_with_reader(
    dir: &Path,
    matches: &impl Fn(&Path) -> bool,
    read: &mut impl FnMut(&Path) -> std::io::Result<Vec<Result<WalkEntry, EntryError>>>,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    walk_files(dir, matches, read, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_files(
    dir: &Path,
    matches: &impl Fn(&Path) -> bool,
    read: &mut impl FnMut(&Path) -> std::io::Result<Vec<Result<WalkEntry, EntryError>>>,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries =
        read(dir).map_err(|error| format!("Cannot read directory `{}`: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| match error {
            EntryError::Read(error) => {
                format!(
                    "Cannot read an entry in directory `{}`: {error}",
                    dir.display()
                )
            }
            EntryError::FileType(path, error) => {
                format!("Cannot inspect `{}`: {error}", path.display())
            }
        })?;
        match entry.kind {
            EntryKind::Directory => {
                if entry
                    .path
                    .file_name()
                    .is_some_and(|name| name.eq_ignore_ascii_case("target"))
                {
                    continue;
                }
                walk_files(&entry.path, matches, read, out)?;
            }
            EntryKind::File if matches(&entry.path) => out.push(entry.path),
            EntryKind::File | EntryKind::Other => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walker_reports_the_directory_for_read_failures() {
        let root = PathBuf::from("unreadable");
        let error = collect_files_with_reader(&root, &|_| true, &mut |_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected denial",
            ))
        })
        .expect_err("walk must fail");

        assert!(error.contains("unreadable"), "error: {error}");
        assert!(error.contains("injected denial"), "error: {error}");
    }

    #[test]
    fn walker_reports_entry_and_file_type_failures() {
        let root = PathBuf::from("root");
        let entry_error = collect_files_with_reader(&root, &|_| true, &mut |_| {
            Ok(vec![Err(EntryError::Read(std::io::Error::other(
                "injected entry failure",
            )))])
        })
        .expect_err("entry failure must abort");
        assert!(entry_error.contains("root"), "error: {entry_error}");

        let bad_path = root.join("unknown.fpas");
        let type_error = collect_files_with_reader(&root, &|_| true, &mut |_| {
            Ok(vec![Err(EntryError::FileType(
                bad_path.clone(),
                std::io::Error::other("injected type failure"),
            ))])
        })
        .expect_err("file type failure must abort");
        assert!(
            type_error.contains(&bad_path.to_string_lossy().to_string()),
            "error: {type_error}"
        );
    }

    #[test]
    fn walker_sorts_files_and_does_not_descend_into_non_directories() {
        let root = PathBuf::from("root");
        let nested = root.join("nested");
        let symlink_like = root.join("linked");
        let mut visited = Vec::new();
        let files =
            collect_files_with_reader(&root, &|path| has_extension(path, "fpas"), &mut |dir| {
                visited.push(dir.to_path_buf());
                if dir == root {
                    return Ok(vec![
                        Ok(WalkEntry {
                            path: root.join("z.fpas"),
                            kind: EntryKind::File,
                        }),
                        Ok(WalkEntry {
                            path: nested.clone(),
                            kind: EntryKind::Directory,
                        }),
                        Ok(WalkEntry {
                            path: symlink_like.clone(),
                            kind: EntryKind::Other,
                        }),
                    ]);
                }
                Ok(vec![Ok(WalkEntry {
                    path: nested.join("a.fpas"),
                    kind: EntryKind::File,
                })])
            })
            .expect("walk succeeds");

        assert_eq!(files, vec![nested.join("a.fpas"), root.join("z.fpas")]);
        assert_eq!(visited, vec![root, nested]);
    }
}
