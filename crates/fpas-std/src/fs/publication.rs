//! Same-directory atomic publication for `Std.Fs.WriteTextAtomic`.

use std::io::{self, Write};
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

pub(super) fn write_text_atomic(path: &Path, text: &str) -> io::Result<()> {
    write_text_atomic_with(path, text, AtomicWriteFile::commit)
}

fn write_text_atomic_with(
    path: &Path,
    text: &str,
    commit: impl FnOnce(AtomicWriteFile) -> io::Result<()>,
) -> io::Result<()> {
    let mut replacement = AtomicWriteFile::open(path)?;
    replacement.write_all(text.as_bytes())?;
    commit(replacement)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{write_text_atomic, write_text_atomic_with};

    fn temp_dir() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        for _ in 0..1024 {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("fpas-fs-publication-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create temporary directory `{}`: {error}", path.display()),
            }
        }
        panic!("could not allocate a unique publication test directory");
    }

    fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
        let mut value = path.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    }

    #[test]
    fn atomic_write_creates_complete_new_file() {
        let root = temp_dir();
        let destination = root.join("note.txt");

        write_text_atomic(&destination, "complete text").expect("publication");

        assert_eq!(
            fs::read_to_string(&destination).expect("published file"),
            "complete text"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn atomic_write_replaces_existing_file_without_leftovers() {
        let root = temp_dir();
        let destination = root.join("note.txt");
        fs::write(&destination, "previous text").expect("previous file");

        write_text_atomic(&destination, "replacement text").expect("replacement");

        assert_eq!(
            fs::read_to_string(&destination).expect("published file"),
            "replacement text"
        );
        assert_eq!(
            fs::read_dir(&root).expect("publication directory").count(),
            1
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn failed_commit_preserves_existing_file_without_restore_phase() {
        let root = temp_dir();
        let destination = root.join("note.txt");
        fs::write(&destination, "previous text").expect("previous file");

        let error = write_text_atomic_with(&destination, "replacement text", |_replacement| {
            Err(std::io::Error::other("injected commit failure"))
        })
        .expect_err("publication must fail");

        assert_eq!(error.to_string(), "injected commit failure");
        assert_eq!(
            fs::read_to_string(&destination).expect("preserved file"),
            "previous text"
        );
        assert_eq!(
            fs::read_dir(&root).expect("publication directory").count(),
            1
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn stale_legacy_siblings_do_not_block_or_get_deleted() {
        let root = temp_dir();
        let destination = root.join("note.txt");
        let stale_temporary = append_suffix(&destination, ".123.1.tmp");
        let stale_backup = append_suffix(&destination, ".123.2.bak");
        fs::write(&stale_temporary, "stale temporary").expect("stale temporary");
        fs::write(&stale_backup, "stale backup").expect("stale backup");

        write_text_atomic(&destination, "new text").expect("publication");

        assert_eq!(
            fs::read_to_string(&destination).expect("published file"),
            "new text"
        );
        assert_eq!(
            fs::read_to_string(&stale_temporary).expect("unowned temporary"),
            "stale temporary"
        );
        assert_eq!(
            fs::read_to_string(&stale_backup).expect("unowned backup"),
            "stale backup"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn legacy_backup_cleanup_cannot_turn_commit_into_failure() {
        let root = temp_dir();
        let destination = root.join("note.txt");
        let legacy_backup = append_suffix(&destination, ".bak");
        fs::create_dir(&legacy_backup).expect("unremovable legacy backup directory");

        write_text_atomic(&destination, "new text").expect("committed publication");

        assert_eq!(
            fs::read_to_string(&destination).expect("published file"),
            "new text"
        );
        assert!(legacy_backup.is_dir());
        fs::remove_dir_all(root).ok();
    }
}
