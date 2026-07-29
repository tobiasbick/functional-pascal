//! Same-directory atomic publication of complete application bundles.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Validate and atomically publish an application executable.
pub fn publish(path: &Path, bytes: &[u8]) -> Result<(), String> {
    crate::decode(bytes).map_err(|error| format!("cannot publish invalid application: {error}"))?;
    let temporary = unique_path(path, ".tmp");
    let backup = unique_path(path, ".bak");
    let mut cleanup = FileCleanup(Some(temporary.clone()));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| io_error("create temporary application", &temporary, error))?;
    file.write_all(bytes)
        .map_err(|error| io_error("write temporary application", &temporary, error))?;
    file.sync_all()
        .map_err(|error| io_error("flush temporary application", &temporary, error))?;
    set_executable(&temporary)?;
    publish_file(path, &temporary, &backup)?;
    cleanup.0 = None;
    Ok(())
}

#[cfg(not(windows))]
fn publish_file(path: &Path, temporary: &Path, _backup: &Path) -> Result<(), String> {
    fs::rename(temporary, path).map_err(|error| io_error("replace application", path, error))
}

#[cfg(windows)]
fn publish_file(path: &Path, temporary: &Path, backup: &Path) -> Result<(), String> {
    let had_previous = path.exists();
    if had_previous {
        fs::rename(path, backup)
            .map_err(|error| io_error("stage previous application", path, error))?;
    }
    if let Err(error) = fs::rename(temporary, path) {
        if had_previous {
            let _ = fs::rename(backup, path);
        }
        return Err(io_error("replace application", path, error));
    }
    if had_previous {
        fs::remove_file(backup)
            .map_err(|error| io_error("remove previous application", backup, error))?;
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| io_error("read application metadata", path, error))?
        .permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(path, permissions)
        .map_err(|error| io_error("mark application executable", path, error))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), String> {
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

fn io_error(operation: &str, path: &Path, error: std::io::Error) -> String {
    format!("failed to {operation} `{}`: {error}", path.display())
}

struct FileCleanup(Option<PathBuf>);

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_file(path);
        }
    }
}
