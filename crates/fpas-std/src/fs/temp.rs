//! `Std.Fs` temporary locations: `TempDir`, `CreateTempDir`, and `CreateTempFile`.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ATTEMPTS: u32 = 64;

/// Host temporary directory with `/` separators.
pub(super) fn temp_dir() -> String {
    slash_path(&std::env::temp_dir())
}

/// Create a new, uniquely named empty directory below the temporary directory.
pub(super) fn create_temp_dir() -> Result<String, String> {
    create_unique("CreateTempDir", |path| fs::create_dir(path))
}

/// Create a new, uniquely named empty file below the temporary directory.
pub(super) fn create_temp_file() -> Result<String, String> {
    create_unique("CreateTempFile", |path| {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(|_| ())
    })
}

fn create_unique(
    operation: &str,
    create: impl Fn(&Path) -> io::Result<()>,
) -> Result<String, String> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let root = std::env::temp_dir();
    for _ in 0..MAX_ATTEMPTS {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.subsec_nanos());
        let candidate = root.join(format!(
            "fpas-{}-{}-{nanos}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match create(&candidate) {
            Ok(()) => return Ok(slash_path(&candidate)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{operation} failed: {error}")),
        }
    }
    Err(format!(
        "{operation} failed: no unique name after {MAX_ATTEMPTS} attempts"
    ))
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}
