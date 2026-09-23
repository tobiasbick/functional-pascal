//! Coordinated same-directory sidecar replacement.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use atomic_write_file::AtomicWriteFile;

use super::SidecarError;

const LOCK_WAIT: Duration = Duration::from_secs(10);
const LOCK_RETRY: Duration = Duration::from_millis(10);

/// Validate staged unit bytes and replace the sidecar under its writer lock.
pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<(), SidecarError> {
    replace_with(
        path,
        bytes,
        |staging, bytes| staging.write_all(bytes),
        AtomicWriteFile::commit,
    )
}

fn replace_with(
    path: &Path,
    bytes: &[u8],
    stage: impl FnOnce(&mut AtomicWriteFile, &[u8]) -> io::Result<()>,
    commit: impl FnOnce(AtomicWriteFile) -> io::Result<()>,
) -> Result<(), SidecarError> {
    let _lock = acquire_lock(path, LockMode::Exclusive, LOCK_WAIT)?;
    let mut staging = AtomicWriteFile::options()
        .read(true)
        .open(path)
        .map_err(|error| SidecarError::Io {
            operation: "create temporary for",
            path: path.to_path_buf(),
            error,
        })?;
    stage(&mut staging, bytes).map_err(|error| SidecarError::Io {
        operation: "write temporary for",
        path: path.to_path_buf(),
        error,
    })?;
    staging.rewind().map_err(|error| SidecarError::Io {
        operation: "seek temporary for",
        path: path.to_path_buf(),
        error,
    })?;
    let mut staged_bytes = Vec::new();
    staging
        .read_to_end(&mut staged_bytes)
        .map_err(|error| SidecarError::Io {
            operation: "read temporary for",
            path: path.to_path_buf(),
            error,
        })?;
    crate::decode(&staged_bytes).map_err(SidecarError::Format)?;
    commit(staging).map_err(|error| SidecarError::Io {
        operation: "replace",
        path: path.to_path_buf(),
        error,
    })
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

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

pub(super) struct LockGuard {
    _file: File,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "filesystem lock fixtures use expect for compact setup"
    )]

    use super::{
        LOCK_WAIT, LockMode, acquire_lock, acquire_read_lock, append_suffix, replace_with,
    };
    use crate::{CompiledUnit, Digest, UnitIdentity};
    use atomic_write_file::AtomicWriteFile;
    use std::fs;
    use std::io::{self, Write};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::Duration;

    fn encoded_unit() -> Vec<u8> {
        let interface = vec![1, 2, 3];
        let object = vec![4, 5, 6];
        crate::encode(&CompiledUnit {
            identity: UnitIdentity {
                unit_name: "demo".to_string(),
                source_hash: Digest::of(b"source"),
                interface_hash: Digest::of(&interface),
                object_hash: Digest::of(&object),
                compiler_version: "test".to_string(),
                bytecode_version: 1,
                options_hash: Digest::of(b"options"),
                dependencies: vec![],
            },
            interface,
            object,
        })
        .expect("valid unit")
    }

    #[test]
    fn failed_staging_and_commit_keep_previous_sidecar() {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("fpas-unit-publication-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).expect("fixture directory");
        let sidecar = root.join("demo.fpascu");
        fs::write(&sidecar, b"previous sidecar").expect("previous sidecar");
        let bytes = encoded_unit();

        let stage_error = replace_with(
            &sidecar,
            &bytes,
            |_staging, _bytes| Err(io::Error::other("injected staging failure")),
            AtomicWriteFile::commit,
        )
        .expect_err("staging must fail");
        assert!(stage_error.to_string().contains("injected staging failure"));
        assert_eq!(
            fs::read(&sidecar).expect("previous sidecar"),
            b"previous sidecar"
        );

        let commit_error = replace_with(
            &sidecar,
            &bytes,
            |staging, bytes| staging.write_all(bytes),
            |_staging| Err(io::Error::other("injected commit failure")),
        )
        .expect_err("commit must fail");
        assert!(commit_error.to_string().contains("injected commit failure"));
        assert_eq!(
            fs::read(&sidecar).expect("previous sidecar"),
            b"previous sidecar"
        );
        assert_eq!(
            fs::read_dir(&root).expect("fixture directory").count(),
            2,
            "only the sidecar and lock may remain"
        );

        replace_with(
            &sidecar,
            &bytes,
            |staging, bytes| staging.write_all(bytes),
            AtomicWriteFile::commit,
        )
        .expect("commit must succeed");
        assert_eq!(fs::read(&sidecar).expect("published sidecar"), bytes);
        fs::remove_dir_all(root).ok();
    }

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
