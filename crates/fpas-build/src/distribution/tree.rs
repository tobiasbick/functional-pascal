use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STAGING_TREE: AtomicU64 = AtomicU64::new(1);

pub(super) fn validate_separate_trees(source: &Path, destination: &Path) -> io::Result<()> {
    let source = canonical_target(source)?;
    let destination = canonical_target(destination)?;
    if source == destination || source.starts_with(&destination) || destination.starts_with(&source)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "standard-library source and distribution directories must be separate, non-nested trees",
        ));
    }
    Ok(())
}

pub(super) fn remove_compiled_unit_artifacts(root: &Path) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            remove_compiled_unit_artifacts(&path)?;
        } else if is_compiled_unit_artifact(&path) {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub(super) fn replace_tree(source: &Path, destination: &Path) -> io::Result<()> {
    let parent = destination.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "distribution directory must have a parent",
        )
    })?;
    fs::create_dir_all(parent)?;
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "distribution directory must have a UTF-8 name",
            )
        })?;
    let id = NEXT_STAGING_TREE.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".{name}.fpas-distribution-{}-{id}",
        std::process::id()
    ));
    if let Err(error) = copy_tree(source, &staging) {
        fs::remove_dir_all(&staging).ok();
        return Err(error);
    }

    if destination.exists() {
        let removal = if destination.is_dir() {
            fs::remove_dir_all(destination)
        } else {
            fs::remove_file(destination)
        };
        if let Err(error) = removal {
            fs::remove_dir_all(&staging).ok();
            return Err(error);
        }
    }
    if let Err(error) = fs::rename(&staging, destination) {
        fs::remove_dir_all(staging).ok();
        return Err(error);
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

fn is_compiled_unit_artifact(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let file_name = file_name.to_ascii_lowercase();
    file_name.ends_with(".fpascu") || file_name.contains(".fpascu.")
}

fn canonical_target(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute.exists() {
        return fs::canonicalize(absolute);
    }

    let mut existing = absolute.as_path();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "path has no existing ancestor")
        })?;
        missing.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "path has no existing ancestor")
        })?;
    }
    let mut resolved = fs::canonicalize(existing)?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}
