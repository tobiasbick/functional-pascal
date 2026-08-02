//! Coordinated same-directory sidecar replacement.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::SidecarError;

const LOCK_WAIT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<(), SidecarError> {
    let _lock = acquire_lock(path, LockMode::Exclusive, LOCK_WAIT)?;
    let temporary = unique_path(path, ".tmp");
    let mut temporary_cleanup = FileCleanup::new(temporary.clone());
    let backup = unique_path(path, ".bak");

    write_complete(&temporary, bytes)?;
    validate_temporary(&temporary)?;
    publish(path, &temporary, &backup)?;
    temporary_cleanup.disarm();
    Ok(())
}

pub(super) fn acquire_read_lock(sidecar: &Path) -> Result<Option<LockGuard>, SidecarError> {
    let lock_path = append_suffix(sidecar, ".lock");
    let file = match File::open(&lock_path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SidecarError::Io {
                operation: "open lock for",
                path: lock_path,
                error,
            });
        }
    };
    wait_for_lock(sidecar, &lock_path, file, LockMode::Shared, LOCK_WAIT).map(Some)
}

fn acquire_lock(sidecar: &Path, mode: LockMode, wait: Duration) -> Result<LockGuard, SidecarError> {
    let lock_path = append_suffix(sidecar, ".lock");
    let started = Instant::now();
    let file = loop {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
        {
            Ok(file) => break file,
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied && lock_path.exists() =>
            {
                if started.elapsed() >= wait {
                    return Err(SidecarError::LockTimeout(sidecar.to_path_buf()));
                }
                std::thread::sleep(LOCK_RETRY);
            }
            Err(error) => {
                return Err(SidecarError::Io {
                    operation: "open lock for",
                    path: lock_path,
                    error,
                });
            }
        }
    };
    wait_for_lock(sidecar, &lock_path, file, mode, wait)
}

fn wait_for_lock(
    sidecar: &Path,
    lock_path: &Path,
    file: File,
    mode: LockMode,
    wait: Duration,
) -> Result<LockGuard, SidecarError> {
    let started = Instant::now();
    loop {
        match mode.try_acquire(&file) {
            Ok(()) => return Ok(LockGuard { _file: file }),
            Err(TryLockError::WouldBlock) => {
                if started.elapsed() >= wait {
                    return Err(SidecarError::LockTimeout(sidecar.to_path_buf()));
                }
                std::thread::sleep(LOCK_RETRY);
            }
            Err(TryLockError::Error(error)) => {
                return Err(SidecarError::Io {
                    operation: "lock",
                    path: lock_path.to_path_buf(),
                    error,
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
enum LockMode {
    Shared,
    Exclusive,
}

impl LockMode {
    fn try_acquire(self, file: &File) -> Result<(), TryLockError> {
        match self {
            Self::Shared => file.try_lock_shared(),
            Self::Exclusive => file.try_lock(),
        }
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

pub(super) struct LockGuard {
    _file: File,
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "filesystem lock fixtures use expect for compact setup"
    )]

    use super::{LOCK_WAIT, LockMode, acquire_lock, acquire_read_lock, append_suffix};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn live_writer_held_beyond_ten_seconds_keeps_its_os_lock() {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let sidecar =
            std::env::temp_dir().join(format!("fpas-unit-lock-{}-{id}.fpascu", std::process::id()));
        let held = acquire_lock(&sidecar, LockMode::Exclusive, LOCK_WAIT).expect("first lock");
        let waiting_sidecar = sidecar.clone();
        let waiting = thread::spawn(move || {
            acquire_lock(
                &waiting_sidecar,
                LockMode::Exclusive,
                LOCK_WAIT + Duration::from_secs(2),
            )
        });

        thread::sleep(LOCK_WAIT + Duration::from_millis(100));

        assert!(!waiting.is_finished(), "second writer entered a live lock");
        drop(held);
        waiting
            .join()
            .expect("waiting writer thread")
            .expect("released lock must be reusable");

        fs::remove_file(append_suffix(&sidecar, ".lock")).ok();
    }

    #[test]
    fn reading_without_a_lock_file_does_not_create_one() {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let sidecar =
            std::env::temp_dir().join(format!("fpas-unit-read-{}-{id}.fpascu", std::process::id()));
        let lock_path = append_suffix(&sidecar, ".lock");

        let lock = acquire_read_lock(&sidecar).expect("missing lock is readable");

        assert!(lock.is_none());
        assert!(!lock_path.exists());
    }
}
