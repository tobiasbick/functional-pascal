//! `Std.Fs` metadata queries: `FileSize`, `ModifiedTime`, and `IsSymLink`.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Size in bytes of the regular file at `path`; directories are rejected.
pub(super) fn file_size(path: &Path) -> Result<i64, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("FileSize failed: {error}"))?;
    if !metadata.is_file() {
        return Err(
            "FileSize failed: path is not a regular file.\n  help: Use Fs.IsFile(Path) first."
                .to_string(),
        );
    }
    i64::try_from(metadata.len()).map_err(|_| "FileSize failed: size exceeds integer".to_string())
}

/// Last modification time of `path` in milliseconds since the Unix epoch.
pub(super) fn modified_time_millis(path: &Path) -> Result<i64, String> {
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|error| format!("ModifiedTime failed: {error}"))?;
    let millis = match modified.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()),
        Err(before) => i64::try_from(before.duration().as_millis()).map(|value| -value),
    };
    millis.map_err(|_| "ModifiedTime failed: timestamp exceeds integer".to_string())
}

/// `true` when `path` itself is a symbolic link, without following it.
pub(super) fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_size_time_and_rejects_directories() {
        let file = super::super::temp::create_temp_file().expect("temp file");
        fs::write(&file, "hello").expect("write");
        assert_eq!(file_size(Path::new(&file)), Ok(5));
        assert!(modified_time_millis(Path::new(&file)).expect("time") > 0);
        assert!(!is_symlink(Path::new(&file)));
        assert!(file_size(&std::env::temp_dir()).is_err());
        assert!(file_size(Path::new("__fpas_missing_size__")).is_err());
        let _ = fs::remove_file(file);
    }
}
