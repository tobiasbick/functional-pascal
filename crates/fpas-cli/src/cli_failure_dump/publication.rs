//! Same-directory staging and atomic new-file publication without replacement.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(1);

/// Publish complete bytes by an atomic hard link; unsupported filesystems report an error.
pub(crate) fn publish(destination: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut staging = create_staging(parent)?;
    let file = staging
        .file
        .as_mut()
        .ok_or_else(|| io::Error::other("Staging file is unavailable"))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(staging.file.take());
    fs::hard_link(&staging.path, destination)?;
    fs::remove_file(&staging.path)?;
    Ok(())
}

fn create_staging(parent: &Path) -> io::Result<Staging> {
    for _ in 0..64 {
        let path = parent.join(format!(
            ".fpas-failure-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                return Ok(Staging {
                    path,
                    file: Some(file),
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Cannot allocate a fresh failure-dump staging file",
    ))
}

struct Staging {
    path: PathBuf,
    file: Option<File>,
}
impl Drop for Staging {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}
