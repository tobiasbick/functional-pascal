//! Platform-aware lexical path containment for workspace discovery.

use std::path::Path;

pub(crate) fn contains(root: &Path, path: &Path) -> bool {
    if cfg!(windows) {
        starts_with_components(path, root, |left, right| left.eq_ignore_ascii_case(right))
    } else {
        path.starts_with(root)
    }
}

pub(super) fn same(left: &Path, right: &Path) -> bool {
    if cfg!(windows) {
        starts_with_components(left, right, |left, right| left.eq_ignore_ascii_case(right))
            && left.components().count() == right.components().count()
    } else {
        left == right
    }
}

fn starts_with_components(path: &Path, root: &Path, equal: impl Fn(&str, &str) -> bool) -> bool {
    let mut path = path.components();
    root.components().all(|component| {
        path.next().is_some_and(|candidate| {
            equal(
                &candidate.as_os_str().to_string_lossy(),
                &component.as_os_str().to_string_lossy(),
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn windows_components_ignore_drive_and_directory_case() {
        assert!(contains(
            Path::new("D:\\Workspace\\Project"),
            Path::new("d:\\workspace\\PROJECT\\src\\missing.fpas")
        ));
        assert!(same(
            Path::new("D:\\Workspace\\Project"),
            Path::new("d:\\workspace\\PROJECT")
        ));
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
