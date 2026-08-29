//! Per-test scratch-directory lifecycle for `fpas test`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// An empty, runner-owned directory removed after one test and its hooks finish.
pub(super) struct TestScratch {
    path: PathBuf,
}

impl TestScratch {
    /// Creates a unique directory below the repository-local scratch root.
    pub(super) fn create(test_path: &Path) -> Result<Self, String> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let root = PathBuf::from(".temp-data");
        fs::create_dir_all(&root).map_err(|error| {
            format!(
                "Error creating test scratch root `{}`: {error}",
                root.display()
            )
        })?;
        let stem = test_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("test");
        let path = root.join(format!(
            "{stem}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).map_err(|error| {
            format!(
                "Error creating test scratch directory `{}`: {error}",
                path.display()
            )
        })?;
        Ok(Self { path })
    }

    /// Returns the directory visible through `Std.Test.ScratchDir()`.
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::TestScratch;

    #[test]
    fn scratch_directories_are_unique_and_removed_on_drop() {
        let first = TestScratch::create(Path::new("first_test.fpas")).unwrap();
        let second = TestScratch::create(Path::new("second_test.fpas")).unwrap();
        let first_path = first.path().to_path_buf();
        let second_path = second.path().to_path_buf();

        assert_ne!(first_path, second_path);
        assert!(first_path.is_dir());
        assert!(second_path.is_dir());

        drop(first);
        drop(second);
        assert!(!first_path.exists());
        assert!(!second_path.exists());
    }

    use std::path::Path;
}
