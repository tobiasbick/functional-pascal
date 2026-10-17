//! Recursive directory operations: `Std.Fs.ReadDirAll` and `Std.Fs.CopyDirAll`.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use std::fs;
use std::path::Path;

use crate::limits::MAX_COLLECTION_LEN;

/// All entries below `path` in stable lexical order, without following directory links.
pub(super) fn read_dir_all_paths(path: &str) -> Result<Vec<String>, String> {
    let root = Path::new(path);
    if !root.is_dir() {
        return Err(format!(
            "ReadDirAll failed: '{path}' is not a directory.\n  help: Use Fs.IsDir(Path) first."
        ));
    }
    let mut paths = Vec::new();
    collect(root, MAX_COLLECTION_LEN as usize, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect(dir: &Path, limit: usize, paths: &mut Vec<String>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| format!("ReadDirAll failed: {error}"))? {
        let entry = entry.map_err(|error| format!("ReadDirAll entry failed: {error}"))?;
        if paths.len() >= limit {
            return Err(format!(
                "ReadDirAll found more than {limit} entries.\n  help: Use a smaller directory tree."
            ));
        }
        let entry_path = entry.path();
        let text = entry_path.to_str().ok_or_else(|| {
            "ReadDirAll entry path is not UTF-8.\n  help: Rename the entry to a valid UTF-8 name."
                .to_string()
        })?;
        paths.push(text.replace(std::path::MAIN_SEPARATOR, "/"));
        let file_type = entry
            .file_type()
            .map_err(|error| format!("ReadDirAll entry failed: {error}"))?;
        if file_type.is_dir() {
            collect(&entry_path, limit, paths)?;
        }
    }
    Ok(())
}

/// Copy the directory tree `source` into `target`, creating directories and replacing files.
pub(super) fn copy_dir_all(source: &Path, target: &Path) -> Result<(), String> {
    if !source.is_dir() {
        return Err(format!(
            "CopyDirAll failed: '{}' is not a directory.\n  help: Use Fs.CopyFile for single files.",
            source.display()
        ));
    }
    if target.starts_with(source) {
        return Err("CopyDirAll failed: target lies inside the source directory".to_string());
    }
    copy_tree(source, target)
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|error| format!("CopyDirAll failed: {error}"))?;
    for entry in fs::read_dir(source).map_err(|error| format!("CopyDirAll failed: {error}"))? {
        let entry = entry.map_err(|error| format!("CopyDirAll entry failed: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("CopyDirAll entry failed: {error}"))?;
        let destination = target.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(format!(
                "CopyDirAll failed: '{}' is a symbolic link; links are not copied",
                entry.path().display()
            ));
        } else if file_type.is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), &destination)
                .map_err(|error| format!("CopyDirAll failed: {error}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
