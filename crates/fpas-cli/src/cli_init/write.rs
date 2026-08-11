//! Conflict detection, idempotency, and rollback for scaffold writes.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use super::plan::{ScaffoldPlan, display_path};

/// Filesystem outcome reported by `fpas init`.
#[derive(Clone, Copy)]
pub(super) enum WriteStatus {
    Planned,
    Created,
    Unchanged,
}

impl WriteStatus {
    /// Returns the stable report value for this filesystem outcome.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Created => "created",
            Self::Unchanged => "unchanged",
        }
    }
}

/// Applies a preflighted plan without overwriting any existing file.
pub(super) fn apply(plan: &ScaffoldPlan) -> Result<WriteStatus, String> {
    validate_root(plan)?;
    let missing_files = preflight_files(plan)?;
    if missing_files.is_empty() {
        return Ok(WriteStatus::Unchanged);
    }

    let missing_directories = missing_directories(plan, &missing_files)?;
    let mut created_directories = Vec::new();
    let mut created_files = Vec::new();
    let result = write_missing(
        plan,
        &missing_directories,
        &missing_files,
        &mut created_directories,
        &mut created_files,
    );
    if let Err(message) = result {
        rollback(&created_files, &created_directories);
        return Err(message);
    }
    Ok(WriteStatus::Created)
}

fn validate_root(plan: &ScaffoldPlan) -> Result<(), String> {
    if plan.root.parent().is_none() {
        return Err(format!(
            "Refusing to initialize at filesystem root `{}`.\n  help: Pass a dedicated directory with `--path <dir>`.",
            plan.root.display()
        ));
    }
    if plan.root.exists() && !plan.root.is_dir() {
        return Err(format!(
            "Cannot initialize `{}` because it is not a directory.\n  help: Choose another `--path`.",
            display_path(&plan.root, &plan.cwd)
        ));
    }
    Ok(())
}

fn preflight_files(plan: &ScaffoldPlan) -> Result<Vec<usize>, String> {
    let mut missing = Vec::new();
    let mut conflicts = Vec::new();
    for (index, file) in plan.files.iter().enumerate() {
        if !file.path.exists() {
            missing.push(index);
            continue;
        }
        match fs::read_to_string(&file.path) {
            Ok(content) if content == file.content => {}
            _ => conflicts.push(display_path(&file.path, &plan.cwd)),
        }
    }
    if conflicts.is_empty() {
        Ok(missing)
    } else {
        Err(format!(
            "Cannot initialize because existing files differ:\n  {}\n  help: Move the files or choose another `--path`; `fpas init` never overwrites files.",
            conflicts.join("\n  ")
        ))
    }
}

fn missing_directories(plan: &ScaffoldPlan, files: &[usize]) -> Result<Vec<PathBuf>, String> {
    let mut directories = Vec::new();
    for index in files {
        let mut current = plan.files[*index].path.parent();
        while let Some(directory) = current {
            if directory.exists() {
                if !directory.is_dir() {
                    return Err(format!(
                        "Cannot create scaffold directory `{}` because that path is a file.",
                        display_path(directory, &plan.cwd)
                    ));
                }
                break;
            }
            if !directories.iter().any(|known| known == directory) {
                directories.push(directory.to_path_buf());
            }
            current = directory.parent();
        }
    }
    directories.sort_by_key(|path| path.components().count());
    Ok(directories)
}

fn write_missing(
    plan: &ScaffoldPlan,
    directories: &[PathBuf],
    files: &[usize],
    created_directories: &mut Vec<PathBuf>,
    created_files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for directory in directories {
        fs::create_dir(directory).map_err(|error| {
            format!(
                "Cannot create scaffold directory `{}`: {error}",
                display_path(directory, &plan.cwd)
            )
        })?;
        created_directories.push(directory.clone());
    }
    for index in files {
        let file = &plan.files[*index];
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file.path)
            .map_err(|error| {
                format!(
                    "Cannot create scaffold file `{}`: {error}",
                    display_path(&file.path, &plan.cwd)
                )
            })?;
        created_files.push(file.path.clone());
        output.write_all(file.content.as_bytes()).map_err(|error| {
            format!(
                "Cannot write scaffold file `{}`: {error}",
                display_path(&file.path, &plan.cwd)
            )
        })?;
    }
    Ok(())
}

fn rollback(files: &[PathBuf], directories: &[PathBuf]) {
    for file in files.iter().rev() {
        let _ = fs::remove_file(file);
    }
    for directory in directories.iter().rev() {
        let _ = fs::remove_dir(directory);
    }
}
