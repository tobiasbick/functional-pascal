use std::fs;
use std::io;
use std::path::Path;

/// Replaces a delivered source tree with an exact copy of its source tree.
pub(crate) fn replace_tree(source: &Path, destination: &Path) -> io::Result<()> {
    if destination.exists() {
        if destination.is_dir() {
            fs::remove_dir_all(destination)?;
        } else {
            fs::remove_file(destination)?;
        }
    }

    copy_tree(source, destination)
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if is_source(&source_path) {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn is_source(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|name| name.to_str()) else {
        return false;
    };
    extension.eq_ignore_ascii_case("fpas") || extension.eq_ignore_ascii_case("fpasprj")
}

#[cfg(test)]
mod tests {
    use super::replace_tree;
    use crate::test_support::{create_temp_dir, write_text};
    use std::fs;

    #[test]
    fn replace_tree_removes_files_missing_from_the_source() {
        let root = create_temp_dir("stdlib-sync");
        let source = root.join("source");
        let destination = root.join("destination");
        write_text(&source.join("Std/Version.fpas"), "current");
        write_text(&destination.join("Std/Version.fpas"), "outdated");
        write_text(&destination.join("Std/Removed.fpas"), "stale");

        replace_tree(&source, &destination).expect("standard library tree must synchronize");

        assert_eq!(
            fs::read_to_string(destination.join("Std/Version.fpas"))
                .expect("copied file must exist"),
            "current"
        );
        assert!(
            !destination.join("Std/Removed.fpas").exists(),
            "files absent from the source must be removed"
        );
        fs::remove_dir_all(&root).expect("temp directory must be removed");
    }

    #[test]
    fn replace_tree_excludes_derived_compiled_unit_artifacts() {
        let root = create_temp_dir("stdlib-sync-artifacts");
        let source = root.join("source");
        let destination = root.join("destination");
        write_text(&source.join("Std/Current.fpas"), "source");
        write_text(&source.join("Std/Current.fpascu"), "stale sidecar");
        write_text(&source.join("Std/Current.fpascu.lock"), "stale lock");
        write_text(&source.join("Std/Current.fpascu.tmp-1"), "stale temporary");
        write_text(&source.join("Cargo.toml"), "build metadata");
        write_text(&source.join("build.rs"), "build script");
        write_text(&source.join("cargo.rs"), "build dependency");

        replace_tree(&source, &destination).expect("standard library tree must synchronize");

        assert!(destination.join("Std/Current.fpas").is_file());
        assert!(!destination.join("Std/Current.fpascu").exists());
        assert!(!destination.join("Std/Current.fpascu.lock").exists());
        assert!(!destination.join("Std/Current.fpascu.tmp-1").exists());
        assert!(!destination.join("Cargo.toml").exists());
        assert!(!destination.join("build.rs").exists());
        assert!(!destination.join("cargo.rs").exists());
        fs::remove_dir_all(&root).expect("temp directory must be removed");
    }
}
