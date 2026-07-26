//! Coordinated same-directory sidecar replacement.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::SidecarError;

const LOCK_WAIT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<(), SidecarError> {
    let lock_path = append_suffix(path, ".lock");
    let _lock = acquire_lock(path, &lock_path)?;
    let temporary = unique_path(path, ".tmp");
    let mut temporary_cleanup = FileCleanup::new(temporary.clone());
    let backup = unique_path(path, ".bak");

    write_complete(&temporary, bytes)?;
    validate_temporary(&temporary)?;
    publish(path, &temporary, &backup)?;
    temporary_cleanup.disarm();
    Ok(())
}

pub(super) fn wait_until_unlocked(sidecar: &Path) -> Result<(), SidecarError> {
    let lock_path = append_suffix(sidecar, ".lock");
    let started = Instant::now();
    while lock_path.exists() {
        remove_stale_lock(&lock_path);
        if !lock_path.exists() {
            break;
        }
        if started.elapsed() >= LOCK_WAIT {
            return Err(SidecarError::LockTimeout(sidecar.to_path_buf()));
        }
        std::thread::sleep(LOCK_RETRY);
    }
    Ok(())
}

fn acquire_lock(sidecar: &Path, lock_path: &Path) -> Result<LockGuard, SidecarError> {
    let started = Instant::now();
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                writeln!(file, "{}", std::process::id()).map_err(|error| SidecarError::Io {
                    operation: "write lock for",
                    path: lock_path.to_path_buf(),
                    error,
                })?;
                return Ok(LockGuard {
                    path: lock_path.to_path_buf(),
                });
            }
            Err(error) if is_lock_contention(&error) => {
                remove_stale_lock(lock_path);
                if started.elapsed() >= LOCK_WAIT {
                    if error.kind() == std::io::ErrorKind::PermissionDenied && !lock_path.exists() {
                        return Err(SidecarError::Io {
                            operation: "lock",
                            path: lock_path.to_path_buf(),
                            error,
                        });
                    }
                    return Err(SidecarError::LockTimeout(sidecar.to_path_buf()));
                }
                std::thread::sleep(LOCK_RETRY);
            }
            Err(error) => {
                return Err(SidecarError::Io {
                    operation: "lock",
                    path: lock_path.to_path_buf(),
                    error,
                });
            }
        }
    }
}

fn is_lock_contention(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::AlreadyExists
        || error.kind() == std::io::ErrorKind::PermissionDenied
}

fn remove_stale_lock(lock_path: &Path) {
    let is_stale = fs::metadata(lock_path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age >= LOCK_WAIT);
    if is_stale {
        let _ = fs::remove_file(lock_path);
    }
}

fn write_complete(path: &Path, bytes: &[u8]) -> Result<(), SidecarError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| SidecarError::Io {
            operation: "create temporary",
            path: path.to_path_buf(),
            error,
        })?;
    file.write_all(bytes).map_err(|error| SidecarError::Io {
        operation: "write temporary",
        path: path.to_path_buf(),
        error,
    })?;
    file.sync_all().map_err(|error| SidecarError::Io {
        operation: "flush temporary",
        path: path.to_path_buf(),
        error,
    })
}

fn validate_temporary(path: &Path) -> Result<(), SidecarError> {
    let bytes = fs::read(path).map_err(|error| SidecarError::Io {
        operation: "read temporary",
        path: path.to_path_buf(),
        error,
    })?;
    crate::decode(&bytes)
        .map(|_| ())
        .map_err(SidecarError::Format)
}

#[cfg(not(windows))]
fn publish(path: &Path, temporary: &Path, _backup: &Path) -> Result<(), SidecarError> {
    fs::rename(temporary, path).map_err(|error| SidecarError::Io {
        operation: "replace",
        path: path.to_path_buf(),
        error,
    })
}

#[cfg(windows)]
fn publish(path: &Path, temporary: &Path, backup: &Path) -> Result<(), SidecarError> {
    let had_previous = path.exists();
    if had_previous {
        fs::rename(path, backup).map_err(|error| SidecarError::Io {
            operation: "stage previous",
            path: path.to_path_buf(),
            error,
        })?;
    }
    if let Err(error) = fs::rename(temporary, path) {
        if had_previous {
            let _ = fs::rename(backup, path);
        }
        return Err(SidecarError::Io {
            operation: "replace",
            path: path.to_path_buf(),
            error,
        });
    }
    if had_previous {
        fs::remove_file(backup).map_err(|error| SidecarError::Io {
            operation: "remove previous",
            path: backup.to_path_buf(),
            error,
        })?;
    }
    Ok(())
}

fn unique_path(path: &Path, suffix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    append_suffix(path, &format!(".{}.{}{suffix}", std::process::id(), id))
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

struct LockGuard {
    path: PathBuf,
}

struct FileCleanup {
    path: Option<PathBuf>,
}

impl FileCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::is_lock_contention;
    use std::io;

    #[test]
    fn permission_denied_during_lock_replacement_is_retried() {
        let error = io::Error::from(io::ErrorKind::PermissionDenied);

        assert!(is_lock_contention(&error));
    }

    #[test]
    fn unrelated_lock_errors_are_reported_immediately() {
        let error = io::Error::from(io::ErrorKind::NotFound);

        assert!(!is_lock_contention(&error));
    }
}
