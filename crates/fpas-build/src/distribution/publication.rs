//! Transactional publication of a completed distribution tree.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::tree::copy_tree_contents;

static NEXT_TRANSACTION_PATH: AtomicU64 = AtomicU64::new(1);

pub(super) fn replace_tree(source: &Path, destination: &Path) -> io::Result<()> {
    replace_tree_with_rename(source, destination, |from, to| fs::rename(from, to))
}

fn replace_tree_with_rename(
    source: &Path,
    destination: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
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
    let staging = create_staging_tree(parent, name)?;
    if let Err(error) = copy_tree_contents(source, &staging) {
        remove_path(&staging).ok();
        return Err(with_path("copy distribution staging tree", &staging, error));
    }

    let backup = if path_exists(destination)? {
        let backup = available_sibling(parent, name, "backup")?;
        if let Err(error) = rename(destination, &backup) {
            remove_path(&staging).ok();
            return Err(with_path("stage previous distribution", destination, error));
        }
        Some(backup)
    } else {
        None
    };

    if let Err(publish_error) = rename(&staging, destination) {
        let restore_error = backup
            .as_ref()
            .and_then(|backup| rename(backup, destination).err());
        let cleanup_error = remove_path(&staging).err();
        return Err(failed_publication(
            destination,
            backup.as_deref(),
            publish_error,
            restore_error,
            cleanup_error,
        ));
    }

    if let Some(backup) = backup
        && let Err(error) = remove_path(&backup)
    {
        return Err(with_path(
            "remove previous distribution after publishing the replacement",
            &backup,
            error,
        ));
    }
    Ok(())
}

fn create_staging_tree(parent: &Path, name: &str) -> io::Result<PathBuf> {
    loop {
        let candidate = transaction_path(parent, name, "staging");
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(with_path(
                    "create distribution staging tree",
                    &candidate,
                    error,
                ));
            }
        }
    }
}

fn available_sibling(parent: &Path, name: &str, role: &str) -> io::Result<PathBuf> {
    loop {
        let candidate = transaction_path(parent, name, role);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(candidate),
            Err(error) => {
                return Err(with_path(
                    "inspect distribution transaction path",
                    &candidate,
                    error,
                ));
            }
        }
    }
}

fn transaction_path(parent: &Path, name: &str, role: &str) -> PathBuf {
    let id = NEXT_TRANSACTION_PATH.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{name}.fpas-distribution-{role}-{}-{id}",
        std::process::id()
    ))
}

fn path_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(with_path("inspect distribution path", path, error)),
    }
}

fn remove_path(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn failed_publication(
    destination: &Path,
    backup: Option<&Path>,
    publish_error: io::Error,
    restore_error: Option<io::Error>,
    cleanup_error: Option<io::Error>,
) -> io::Error {
    let kind = publish_error.kind();
    let mut detail = format!(
        "cannot publish staged distribution `{}`: {publish_error}",
        destination.display()
    );
    if let Some(restore_error) = restore_error {
        let backup = backup.map_or_else(
            || "<missing backup>".to_string(),
            |path| path.display().to_string(),
        );
        detail.push_str(&format!(
            "; additionally failed to restore previous distribution from `{backup}`: {restore_error}"
        ));
    }
    if let Some(cleanup_error) = cleanup_error {
        detail.push_str(&format!(
            "; additionally failed to remove the staging tree: {cleanup_error}"
        ));
    }
    io::Error::new(kind, detail)
}

fn with_path(action: &str, path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("cannot {action} `{}`: {error}", path.display()),
    )
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "publication filesystem fixtures use direct assertions for diagnostic clarity"
    )]

    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn temp_dir() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "fpas-distribution-publication-{}-{id}",
            std::process::id()
        ))
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent");
        }
        fs::write(path, contents).expect("fixture file");
    }

    #[test]
    fn failed_publish_restores_the_previous_distribution() {
        let root = temp_dir();
        let source = root.join("source");
        let destination = root.join("distribution");
        write(&source.join("new.txt"), "new");
        write(&destination.join("old.txt"), "old");
        let mut renames = 0;

        replace_tree_with_rename(&source, &destination, |from, to| {
            renames += 1;
            if renames == 2 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected publish failure",
                ));
            }
            fs::rename(from, to)
        })
        .expect_err("publishing must fail");

        assert_eq!(
            fs::read_to_string(destination.join("old.txt")).expect("restored distribution"),
            "old"
        );
        assert!(!destination.join("new.txt").exists());
        assert!(
            fs::read_dir(&root)
                .expect("fixture root")
                .map(|entry| entry.expect("fixture entry").file_name())
                .all(|name| !name.to_string_lossy().contains("fpas-distribution-"))
        );
        fs::remove_dir_all(root).expect("fixture cleanup");
    }

    #[test]
    fn failed_restore_reports_the_backup_that_preserves_the_previous_distribution() {
        let root = temp_dir();
        let source = root.join("source");
        let destination = root.join("distribution");
        write(&source.join("new.txt"), "new");
        write(&destination.join("old.txt"), "old");
        let mut renames = 0;

        let error = replace_tree_with_rename(&source, &destination, |from, to| {
            renames += 1;
            if renames >= 2 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("injected rename failure from {}", from.display()),
                ));
            }
            fs::rename(from, to)
        })
        .expect_err("publishing and restoration must fail");

        let backup = fs::read_dir(&root)
            .expect("fixture root")
            .map(|entry| entry.expect("fixture entry").path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains("fpas-distribution-backup"))
            })
            .expect("preserved backup");
        assert!(error.to_string().contains("failed to restore"));
        assert!(!destination.exists());
        assert_eq!(
            fs::read_to_string(backup.join("old.txt")).expect("preserved distribution backup"),
            "old"
        );
        assert_eq!(
            fs::read_dir(&root)
                .expect("fixture root")
                .map(|entry| entry.expect("fixture entry").file_name())
                .filter(|name| name.to_string_lossy().contains("fpas-distribution-"))
                .count(),
            1
        );
        fs::remove_dir_all(root).expect("fixture cleanup");
    }
}
