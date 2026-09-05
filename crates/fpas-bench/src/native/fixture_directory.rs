//! Exclusively owned scratch directories for native benchmark fixtures.

use std::path::{Path, PathBuf};

/// A newly created directory removed when its workload finishes.
pub(super) struct FixtureDirectory(PathBuf);

impl FixtureDirectory {
    /// Creates one owned directory beneath the workspace benchmark scratch root.
    pub(super) fn create(root: &Path) -> Result<Self, String> {
        let parent = root.join(".temp-data/bench/native-fixtures");
        std::fs::create_dir_all(&parent).map_err(|error| error.to_string())?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let path = parent.join(format!("{}-{nonce}", std::process::id()));
        std::fs::create_dir(&path).map_err(|error| error.to_string())?;
        Ok(Self(path))
    }

    /// Returns the exclusively owned directory path.
    pub(super) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
