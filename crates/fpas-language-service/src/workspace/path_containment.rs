//! Platform-aware lexical path containment for workspace discovery.

use std::path::Path;

pub(crate) fn contains(root: &Path, path: &Path) -> bool {
    if cfg!(windows) {
        windows_contains(root, path)
    } else {
        path.starts_with(root)
    }
}

pub(super) fn same(left: &Path, right: &Path) -> bool {
    if cfg!(windows) {
        windows_same(left, right)
    } else {
        left == right
    }
}

fn windows_contains(root: &Path, path: &Path) -> bool {
    let Ok(root) = std::fs::canonicalize(root) else {
        return path.starts_with(root);
    };
    let Some(existing_ancestor) = canonical_existing_ancestor(path) else {
        return false;
    };
    existing_ancestor.starts_with(root)
}

fn windows_same(left: &Path, right: &Path) -> bool {
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn canonical_existing_ancestor(path: &Path) -> Option<std::path::PathBuf> {
    let mut candidate = path.to_path_buf();
    loop {
        if let Ok(canonical) = std::fs::canonicalize(&candidate) {
            return Some(canonical);
        }
        if !candidate.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    use std::path::PathBuf;

    #[cfg(windows)]
    #[test]
    fn windows_components_ignore_drive_and_directory_case() {
        let base = std::env::temp_dir().join(format!(
            "fpas-path-containment-ascii-{}",
            std::process::id()
        ));
        let root = base.join("Workspace/Project");
        std::fs::create_dir_all(&root).expect("ASCII root directory");
        let differently_cased_root = PathBuf::from(root.to_string_lossy().to_ascii_lowercase());

        assert!(contains(
            &root,
            &differently_cased_root.join("src/missing.fpas")
        ));
        assert!(same(&root, &differently_cased_root));

        std::fs::remove_dir_all(&base).ok();
    }

    #[cfg(windows)]
    #[test]
    fn windows_components_use_native_unicode_case_matching() {
        let base = std::env::temp_dir().join(format!(
            "fpas-path-containment-unicode-{}",
            std::process::id()
        ));
        let root = base.join("Ärea");
        std::fs::create_dir_all(&root).expect("Unicode root directory");
        let differently_cased =
            PathBuf::from(root.to_string_lossy().to_lowercase()).join("missing.fpas");

        let contained = contains(&root, &differently_cased);
        std::fs::remove_dir_all(&base).ok();

        assert!(contained);
    }

    #[cfg(windows)]
    #[test]
    fn windows_components_do_not_merge_distinct_unpaired_surrogates() {
        use std::os::windows::ffi::OsStringExt;

        let left = PathBuf::from(std::ffi::OsString::from_wide(&[0xd800]));
        let right = PathBuf::from(std::ffi::OsString::from_wide(&[0xd801]));

        assert!(!same(&left, &right));
    }

    #[cfg(not(windows))]
    #[test]
    fn native_components_remain_case_sensitive() {
        assert!(!contains(
            Path::new("/workspace/Project"),
            Path::new("/workspace/project/missing.fpas")
        ));
        assert!(!same(
            Path::new("/workspace/Project"),
            Path::new("/workspace/project")
        ));
    }
}
