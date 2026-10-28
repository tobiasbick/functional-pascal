//! Recursive directory operations: `Std.Io.Fs.ReadDirAll` and `Std.Io.Fs.CopyDirAll`.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use super::failure::Failure;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::stdlib::limits::MAX_COLLECTION_LEN;

/// All entries below `path` in stable lexical order, without following directory links.
pub(super) fn read_dir_all_paths(path: &str) -> Result<Vec<String>, Failure> {
    let root = Path::new(path);
    let mut paths = Vec::new();
    collect(root, MAX_COLLECTION_LEN as usize, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect(dir: &Path, limit: usize, paths: &mut Vec<String>) -> Result<(), Failure> {
    for entry in fs::read_dir(dir).map_err(|error| Failure::at(dir, error))? {
        let entry = entry.map_err(|error| Failure::at(dir, error))?;
        if paths.len() >= limit {
            return Err(Failure::LimitExceeded("Std.Io.Fs.ReadDirAll", limit as i64));
        }
        let entry_path = entry.path();
        let text = entry_path.to_str().ok_or_else(|| {
            Failure::EncodingAtPath(
                entry_path.to_string_lossy().into_owned(),
                "ReadDirAll entry path is not UTF-8".into(),
            )
        })?;
        paths.push(text.replace(std::path::MAIN_SEPARATOR, "/"));
        let file_type = entry
            .file_type()
            .map_err(|error| Failure::at(&entry.path(), error))?;
        if file_type.is_dir() {
            collect(&entry_path, limit, paths)?;
        }
    }
    Ok(())
}

/// Copy the directory tree `source` into `target`, creating directories and replacing files.
/// Reject targets resolving to the source or one of its descendants before copying.
pub(super) fn copy_dir_all(source: &Path, target: &Path) -> Result<(), Failure> {
    if !fs::metadata(source)?.is_dir() {
        return Err(Failure::InvalidArgument(
            "Source",
            "source is not a directory; use CopyFile for single files".into(),
        ));
    }
    let source_path = fs::canonicalize(source)?;
    copy_tree(source, target, &source_path)
}

fn resolve_target_path(target: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(target)?;
    let mut ancestor = absolute.as_path();
    let mut missing = Vec::new();
    let mut resolved = loop {
        match canonicalize_target_prefix(ancestor) {
            Ok(path) => break path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(parent) = ancestor.parent() else {
                    return Err(error);
                };
                let Some(component) = ancestor.components().next_back() else {
                    return Err(error);
                };
                missing.push(component);
                ancestor = parent;
            }
            Err(error) => return Err(error),
        }
    };
    // Resolve existing prefixes first: a directory link followed by `..` uses
    // filesystem semantics, while a missing suffix will be created as directories.
    for component in missing.into_iter().rev() {
        let candidate = resolved.join(component.as_os_str());
        match canonicalize_target_prefix(&candidate) {
            Ok(path) => resolved = path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => match component {
                Component::ParentDir => {
                    resolved.pop();
                }
                Component::CurDir => {}
                Component::Normal(_) => resolved = candidate,
                _ => return Err(error),
            },
            Err(error) => return Err(error),
        }
    }
    Ok(resolved)
}

fn canonicalize_target_prefix(path: &Path) -> io::Result<PathBuf> {
    let resolved = fs::canonicalize(path);
    if matches!(&resolved, Err(error) if error.kind() == io::ErrorKind::NotFound)
        && fs::symlink_metadata(path)?.file_type().is_symlink()
    {
        // A dangling link is existing storage, not a missing directory suffix.
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "target contains an unresolved symbolic link",
        ));
    }
    resolved
}

fn resolve_copy_target(source_root: &Path, target: &Path) -> Result<PathBuf, Failure> {
    let target_path = resolve_target_path(target).map_err(|error| {
        if error.kind() == io::ErrorKind::InvalidInput && error.raw_os_error().is_none() {
            Failure::InvalidArgument("Target", error.to_string())
        } else {
            Failure::at(target, error)
        }
    })?;
    if target_path.starts_with(source_root) {
        return Err(Failure::InvalidArgument(
            "Target",
            "CopyDirAll failed: target lies inside the source directory".into(),
        ));
    }
    Ok(target_path)
}

fn copy_tree(source: &Path, target: &Path, source_root: &Path) -> Result<(), Failure> {
    let resolved_target = resolve_copy_target(source_root, target)?;
    fs::create_dir_all(&resolved_target).map_err(|error| Failure::at(target, error))?;
    for entry in fs::read_dir(source).map_err(|error| Failure::at(source, error))? {
        let entry = entry.map_err(|error| Failure::at(source, error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| Failure::at(&entry.path(), error))?;
        let destination = resolved_target.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(Failure::InvalidArgument(
                "Source",
                format!(
                    "CopyDirAll failed: '{}' is a symbolic link; links are not copied",
                    entry.path().display()
                ),
            ));
        } else if file_type.is_dir() {
            copy_tree(&entry.path(), &destination, source_root)?;
        } else {
            let destination = resolve_copy_target(source_root, &destination)?;
            fs::copy(entry.path(), destination)
                .map_err(|error| Failure::at(&entry.path(), error))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            Self(PathBuf::from(
                super::super::temp::create_temp_dir().expect("temp directory"),
            ))
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn copy_rejects_inside_target_with_relative_and_absolute_paths() {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.temp-data");
        fs::create_dir_all(&scratch).expect("scratch directory");
        let name = format!(
            "copy-dir-containment-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let root = scratch.join(&name);
        let source = root.join("src");
        let target = source.join("copy");
        fs::create_dir_all(&target).expect("fixture directories");
        // If containment is missed, the first recursive destination is this file.
        fs::write(target.join("copy"), "keep").expect("bounded recursion blocker");
        let absolute_source = fs::canonicalize(&source).expect("absolute source");
        let absolute_target = absolute_source.join("copy");
        let relative_source = Path::new("../../.temp-data").join(name).join("src");
        let relative_target = relative_source.join("copy");
        let results = [
            copy_dir_all(&relative_source, &absolute_target),
            copy_dir_all(&absolute_source, &relative_target),
        ];
        let contents = fs::read_to_string(target.join("copy")).expect("blocker survives");
        let entries = fs::read_dir(&target).expect("target entries").count();
        fs::remove_dir_all(&root).expect("remove fixture");

        for result in results {
            assert!(matches!(result, Err(Failure::InvalidArgument("Target", _))));
        }
        assert_eq!(contents, "keep");
        assert_eq!(entries, 1);
    }

    #[test]
    fn resolves_missing_target_suffixes_without_creating_them() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        fs::create_dir(&source).expect("source");
        let canonical = fs::canonicalize(&source).expect("canonical source");

        for suffix in ["new/deep", "new/../copy/deep", "new/../../outside/deep"] {
            let target = source.join(suffix);
            let expected = match suffix {
                "new/deep" => canonical.join("new/deep"),
                "new/../copy/deep" => canonical.join("copy/deep"),
                _ => canonical
                    .parent()
                    .expect("source parent")
                    .join("outside/deep"),
            };
            assert_eq!(
                resolve_target_path(&target).expect("resolve target"),
                expected
            );
        }
        assert_eq!(fs::read_dir(&source).expect("source entries").count(), 0);
    }

    #[test]
    fn copy_rejects_parent_alias_before_changing_existing_target() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        let target = source.join("copy");
        fs::create_dir_all(&target).expect("source directories");
        fs::create_dir(root.0.join("outside")).expect("alias prefix");
        fs::write(target.join("copy"), "keep").expect("bounded recursion blocker");

        let result = copy_dir_all(&source, &root.0.join("outside/../src/copy"));

        assert!(matches!(result, Err(Failure::InvalidArgument("Target", _))));
        assert_eq!(
            fs::read_to_string(target.join("copy")).expect("blocker"),
            "keep"
        );
        assert_eq!(fs::read_dir(&target).expect("target entries").count(), 1);
    }

    #[test]
    fn copy_resolves_missing_parent_segments_without_changing_source() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        fs::create_dir(&source).expect("source directory");
        fs::write(source.join("value.txt"), "original").expect("source file");
        let target = source.join("missing/../../outside/copy");

        copy_dir_all(&source, &target).expect("copy outside source");

        assert_eq!(
            fs::read_to_string(root.0.join("outside/copy/value.txt")).expect("copied file"),
            "original"
        );
        assert!(!source.join("missing").exists());
        assert_eq!(fs::read_dir(&source).expect("source entries").count(), 1);
    }

    #[cfg(unix)]
    fn directory_link(original: &Path, link: &Path) {
        std::os::unix::fs::symlink(original, link).expect("directory link");
    }

    #[cfg(windows)]
    fn directory_link(original: &Path, link: &Path) {
        let result = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(std::path::absolute(link).expect("absolute junction path"))
            .arg(std::path::absolute(original).expect("absolute junction target"))
            .output()
            .expect("create directory junction");
        assert!(
            result.status.success(),
            "directory junction creation failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn copy_rejects_target_ancestor_alias_into_source() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        let target = source.join("copy");
        fs::create_dir_all(&target).expect("source directories");
        fs::write(target.join("copy"), "keep").expect("bounded recursion blocker");
        let alias = root.0.join("alias");
        directory_link(&source, &alias);

        let result = copy_dir_all(&source, &alias.join("copy"));

        assert!(matches!(result, Err(Failure::InvalidArgument("Target", _))));
        assert_eq!(
            fs::read_to_string(target.join("copy")).expect("blocker"),
            "keep"
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn copy_rejects_destination_directory_alias_into_source() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        let nested = source.join("copy");
        let target = root.0.join("target");
        fs::create_dir_all(&nested).expect("source directories");
        fs::create_dir(&target).expect("target directory");
        fs::write(nested.join("copy"), "keep").expect("bounded fixture file");
        directory_link(&nested, &target.join("copy"));

        let result = copy_dir_all(&source, &target);

        assert!(matches!(result, Err(Failure::InvalidArgument("Target", _))));
        assert_eq!(
            fs::read_to_string(nested.join("copy")).expect("source file"),
            "keep"
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn copy_preserves_external_target_aliases_and_parent_resolution() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        let outside = root.0.join("outside");
        fs::create_dir(&source).expect("source");
        fs::create_dir_all(outside.join("deep")).expect("external directories");
        fs::write(source.join("value.txt"), "original").expect("source file");
        let alias = root.0.join("alias");
        directory_link(&outside.join("deep"), &alias);
        let target = alias.join("../copy");

        copy_dir_all(&source, &target).expect("external alias copy");

        assert_eq!(
            fs::read_to_string(target.join("value.txt")).expect("copied file"),
            "original"
        );
        let expected = fs::canonicalize(target.parent().expect("target parent"))
            .expect("actual operating-system parent")
            .join("copy");
        assert_eq!(
            resolve_target_path(&target).expect("resolve target"),
            expected
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_rejects_destination_file_alias_into_source() {
        let root = TestDirectory::new();
        let source = root.0.join("src");
        let target = root.0.join("target");
        fs::create_dir(&source).expect("source");
        fs::create_dir(&target).expect("target");
        fs::write(source.join("value.txt"), "original").expect("source file");
        for (destination, message) in [
            (
                "value.txt",
                "CopyDirAll failed: target lies inside the source directory",
            ),
            (
                "unexpected.txt",
                "target contains an unresolved symbolic link",
            ),
        ] {
            let link = target.join("value.txt");
            std::os::unix::fs::symlink(source.join(destination), &link)
                .expect("destination file link");

            assert_eq!(
                copy_dir_all(&source, &target)
                    .expect_err("reject target")
                    .to_string(),
                message
            );
            assert_eq!(
                fs::read_to_string(source.join("value.txt")).expect("source file"),
                "original"
            );
            assert!(!source.join("unexpected.txt").exists());
            fs::remove_file(link).expect("remove destination link");
        }
    }

    #[test]
    fn copies_and_lists_nested_tree_in_order() {
        let root =
            Path::new(&super::super::temp::create_temp_dir().expect("temp dir")).to_path_buf();
        let source = root.join("src");
        fs::create_dir_all(source.join("a/empty")).expect("fixture dirs");
        fs::write(source.join("a/file.txt"), "x").expect("fixture file");
        fs::write(source.join("b.txt"), "y").expect("fixture file");

        let target = root.join("copy");
        copy_dir_all(&source, &target).expect("copy");
        let listed = read_dir_all_paths(target.to_str().expect("UTF-8")).expect("list");
        let prefix = target
            .to_str()
            .expect("UTF-8")
            .replace(std::path::MAIN_SEPARATOR, "/");
        let relative: Vec<_> = listed
            .iter()
            .map(|path| path.strip_prefix(&prefix).expect("below target"))
            .collect();
        assert_eq!(relative, ["/a", "/a/empty", "/a/file.txt", "/b.txt"]);

        assert!(copy_dir_all(&source, &source.join("inner")).is_err());
        assert!(read_dir_all_paths(source.join("b.txt").to_str().expect("UTF-8")).is_err());
        let _ = fs::remove_dir_all(root);
    }
}
