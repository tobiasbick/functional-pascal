//! Stable, bounded immediate directory enumeration for `Std.Fs.ReadDir`.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md`

use std::fs;
use std::path::Path;

use crate::limits::MAX_COLLECTION_LEN;

pub(super) fn read_dir_paths(path: &str) -> Result<Vec<String>, String> {
    read_dir_paths_with_limit(Path::new(path), MAX_COLLECTION_LEN as usize)
}

fn read_dir_paths_with_limit(path: &Path, limit: usize) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| format!("ReadDir failed: {error}"))? {
        let entry = entry.map_err(|error| format!("ReadDir entry failed: {error}"))?;
        if paths.len() >= limit {
            return Err(format!(
                "ReadDir contains more than {limit} entries.\n  help: Use a smaller directory."
            ));
        }
        let entry_path = entry.path();
        let text = entry_path.to_str().ok_or_else(|| {
            "ReadDir entry path is not UTF-8.\n  help: Rename the entry to a valid UTF-8 name."
                .to_string()
        })?;
        paths.push(text.replace(std::path::MAIN_SEPARATOR, "/"));
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Directory(PathBuf);

    impl Directory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(1);
            let path = std::env::temp_dir().join(format!(
                "fpas-read-dir-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("isolated fixture directory");
            Self(path)
        }
    }

    impl Drop for Directory {
        fn drop(&mut self) {
            assert!(self.0.starts_with(std::env::temp_dir()));
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn lists_files_empty_directories_and_unicode_in_order_without_recursion() {
        let root = Directory::new();
        assert!(
            read_dir_paths_with_limit(&root.0, 0)
                .expect("empty directory")
                .is_empty()
        );
        fs::create_dir(root.0.join("b empty")).expect("empty child");
        fs::create_dir(root.0.join("a child")).expect("nested child");
        fs::write(root.0.join("a child/nested.txt"), "nested").expect("nested file");
        fs::write(root.0.join("ü.txt"), "unicode").expect("unicode file");
        let expected = ["a child", "b empty", "ü.txt"].map(|name| {
            root.0
                .join(name)
                .to_str()
                .expect("UTF-8")
                .replace('\\', "/")
        });
        assert_eq!(
            read_dir_paths_with_limit(&root.0, 3).expect("listing"),
            expected
        );
        assert!(read_dir_paths_with_limit(&root.0, 2).is_err());
    }

    #[test]
    fn missing_directory_and_file_are_errors() {
        let root = Directory::new();
        assert!(read_dir_paths_with_limit(&root.0.join("missing"), 10).is_err());
        fs::write(root.0.join("file"), "file").expect("file");
        assert!(read_dir_paths_with_limit(&root.0.join("file"), 10).is_err());
        assert!(read_dir_paths("").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn native_backslash_in_an_entry_name_remains_usable() {
        let root = Directory::new();
        let path = root.0.join("a\\b.txt");
        fs::write(&path, "kept").expect("backslash filename");
        let entries = read_dir_paths_with_limit(&root.0, 1).expect("listing");
        assert_eq!(entries, vec![path.to_str().expect("UTF-8").to_string()]);
        assert_eq!(
            fs::read_to_string(&entries[0]).expect("returned path"),
            "kept"
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_entry_is_rejected_without_lossy_aliasing() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let root = Directory::new();
        fs::write(root.0.join(OsString::from_vec(vec![0xff])), "file").expect("non-UTF-8 entry");
        assert!(
            read_dir_paths_with_limit(&root.0, 10)
                .expect_err("must reject")
                .contains("not UTF-8")
        );
    }
}
