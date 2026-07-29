//! Same-directory validated replacement for compiled program images.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const LOCK_WAIT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

pub(super) fn read(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let lock_path = append_suffix(path, ".lock");
    wait_until_unlocked(path, &lock_path)?;
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error("read", path, error)),
    }
}

pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
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

fn wait_until_unlocked(path: &Path, lock_path: &Path) -> Result<(), String> {
    let started = Instant::now();
    while lock_path.exists() {
        remove_stale_lock(lock_path);
        if !lock_path.exists() {
            break;
        }
        if started.elapsed() >= LOCK_WAIT {
            return Err(lock_timeout(path));
        }
        std::thread::sleep(LOCK_RETRY);
    }
    Ok(())
}

fn acquire_lock(path: &Path, lock_path: &Path) -> Result<LockGuard, String> {
    let started = Instant::now();
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                writeln!(file, "{}", std::process::id())
                    .map_err(|error| io_error("write lock for", lock_path, error))?;
                return Ok(LockGuard {
                    path: lock_path.to_path_buf(),
                });
            }
            Err(error) if is_lock_contention(&error) => {
                remove_stale_lock(lock_path);
                if started.elapsed() >= LOCK_WAIT {
                    if error.kind() == io::ErrorKind::PermissionDenied && !lock_path.exists() {
                        return Err(io_error("lock", lock_path, error));
                    }
                    return Err(lock_timeout(path));
                }
                std::thread::sleep(LOCK_RETRY);
            }
            Err(error) => return Err(io_error("lock", lock_path, error)),
        }
    }
}

fn is_lock_contention(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::AlreadyExists || error.kind() == io::ErrorKind::PermissionDenied
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

fn write_complete(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error("create temporary", path, error))?;
    file.write_all(bytes)
        .map_err(|error| io_error("write temporary", path, error))?;
    file.sync_all()
        .map_err(|error| io_error("flush temporary", path, error))
}

fn validate_temporary(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| io_error("read temporary", path, error))?;
    fpas_program::decode(&bytes).map(|_| ()).map_err(|error| {
        format!(
            "temporary compiled program `{}` is invalid: {error}",
            path.display()
        )
    })
}

#[cfg(not(windows))]
fn publish(path: &Path, temporary: &Path, _backup: &Path) -> Result<(), String> {
    fs::rename(temporary, path).map_err(|error| io_error("replace", path, error))
}

#[cfg(windows)]
fn publish(path: &Path, temporary: &Path, backup: &Path) -> Result<(), String> {
    let had_previous = path.exists();
    if had_previous {
        fs::rename(path, backup).map_err(|error| io_error("stage previous", path, error))?;
    }
    if let Err(error) = fs::rename(temporary, path) {
        if had_previous {
            let _ = fs::rename(backup, path);
        }
        return Err(io_error("replace", path, error));
    }
    if had_previous {
        fs::remove_file(backup).map_err(|error| io_error("remove previous", backup, error))?;
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

fn io_error(operation: &str, path: &Path, error: io::Error) -> String {
    format!(
        "failed to {operation} compiled program `{}`: {error}",
        path.display()
    )
}

fn lock_timeout(path: &Path) -> String {
    format!(
        "timed out waiting to replace compiled program `{}`; another compiler process may still be writing it",
        path.display()
    )
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

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}
