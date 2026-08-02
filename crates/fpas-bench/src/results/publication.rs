//! Durable same-directory publication for benchmark text artifacts.

use atomic_write_file::AtomicWriteFile;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Write complete text to a temporary sibling and atomically replace `path`.
pub(super) fn write_text(path: &Path, text: &str) -> Result<(), String> {
    write_text_with(
        path,
        text,
        |file, bytes| file.write_all(bytes),
        AtomicWriteFile::commit,
    )
}

fn write_text_with(
    path: &Path,
    text: &str,
    write: impl FnOnce(&mut AtomicWriteFile, &[u8]) -> io::Result<()>,
    commit: impl FnOnce(AtomicWriteFile) -> io::Result<()>,
) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        format!(
            "cannot publish benchmark artifact without a parent: {}",
            path.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| io_error("create artifact directory", parent, error))?;
    let mut staging = AtomicWriteFile::open(path)
        .map_err(|error| io_error("create staged artifact", path, error))?;
    write(&mut staging, text.as_bytes())
        .map_err(|error| io_error("write staged artifact", path, error))?;
    staging
        .sync_all()
        .map_err(|error| io_error("flush staged artifact", path, error))?;
    commit(staging).map_err(|error| io_error("publish staged artifact", path, error))
}

fn io_error(action: &str, path: &Path, error: io::Error) -> String {
    format!("cannot {action} `{}`: {error}", path.display())
}

#[cfg(test)]
mod tests {
    use super::{write_text, write_text_with};
    use std::error::Error;
    use std::fs;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new() -> io::Result<Self> {
            static NEXT_ID: AtomicU64 = AtomicU64::new(1);
            loop {
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "fpas-bench-publication-{}-{id}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Ok(Self(path)),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error),
                }
            }
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _cleanup_result = fs::remove_dir_all(&self.0);
        }
    }

    fn artifact_state(root: &Path, destination: &Path) -> io::Result<(String, usize)> {
        let text = fs::read_to_string(destination)?;
        let sibling_count = fs::read_dir(root)?.filter_map(Result::ok).count();
        Ok((text, sibling_count))
    }

    #[test]
    fn write_failure_preserves_previous_artifact() -> Result<(), Box<dyn Error>> {
        let root = TempRoot::new()?;
        let destination = root.path().join("history.md");
        fs::write(&destination, "previous")?;
        let result = write_text_with(
            &destination,
            "replacement",
            |_file, _bytes| Err(io::Error::new(io::ErrorKind::WriteZero, "injected write")),
            atomic_write_file::AtomicWriteFile::commit,
        );

        assert_eq!(
            (result.is_err(), artifact_state(root.path(), &destination)?),
            (true, ("previous".to_owned(), 1))
        );
        Ok(())
    }

    #[test]
    fn publish_failure_preserves_previous_artifact() -> Result<(), Box<dyn Error>> {
        let root = TempRoot::new()?;
        let destination = root.path().join("history.md");
        fs::write(&destination, "previous")?;
        let result = write_text_with(
            &destination,
            "replacement",
            |file, bytes| file.write_all(bytes),
            |_file| {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected publish",
                ))
            },
        );

        assert_eq!(
            (result.is_err(), artifact_state(root.path(), &destination)?),
            (true, ("previous".to_owned(), 1))
        );
        Ok(())
    }

    #[test]
    fn successful_publication_replaces_previous_artifact() -> Result<(), Box<dyn Error>> {
        let root = TempRoot::new()?;
        let destination = root.path().join("snapshot.json");
        fs::write(&destination, "previous")?;

        write_text(&destination, "replacement")?;

        assert_eq!(
            artifact_state(root.path(), &destination)?,
            ("replacement".to_owned(), 1)
        );
        Ok(())
    }
}
