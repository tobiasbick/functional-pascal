//! Read-only validation; success and pre-execution errors create no directories or files.

use std::{fs, io, path::Path};

/// Reject an existing destination, including a dangling symlink.
pub(crate) fn validate(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(format!(
            "Failure dump destination `{}` already exists.\n  help: Choose a new file path.",
            path.display()
        )),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(format!(
            "Cannot inspect failure dump destination `{}`: {error}",
            path.display()
        )),
    }
}
