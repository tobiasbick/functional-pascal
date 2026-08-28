//! OS-locked, same-directory atomic replacement for compiled program images.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;

#[cfg(test)]
pub(super) fn replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    PublicationLock::acquire(path)?.prepare(bytes)?.commit()
}

fn validate_image(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fpas_program::decode(bytes).map(|_| ()).map_err(|error| {
        format!(
            "temporary compiled program `{}` is invalid: {error}",
            path.display()
        )
    })
}

fn io_error(operation: &str, path: &Path, error: io::Error) -> String {
    format!(
        "failed to {operation} compiled program `{}`: {error}",
        path.display()
    )
}

/// Exclusive transaction guard for one compiled program image path.
pub(super) struct PublicationLock {
    path: PathBuf,
    _file: File,
}

impl PublicationLock {
    /// Acquires the persistent sidecar lock associated with `path`.
    pub(super) fn acquire(path: &Path) -> Result<Self, String> {
        let lock_path = append_suffix(path, ".lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| io_error("open publication lock for", &lock_path, error))?;
        file.lock()
            .map_err(|error| io_error("lock", &lock_path, error))?;
        Ok(Self {
            path: path.to_path_buf(),
            _file: file,
        })
    }

    /// Reads the current image while retaining exclusive publication ownership.
    pub(super) fn read(&self) -> Result<Option<Vec<u8>>, String> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io_error("read", &self.path, error)),
        }
    }

    /// Validates and writes a temporary image while retaining the lock.
    pub(super) fn prepare(&self, bytes: &[u8]) -> Result<PendingReplacement<'_>, String> {
        validate_image(&self.path, bytes)?;
        let mut replacement = AtomicWriteFile::open(&self.path)
            .map_err(|error| io_error("create temporary for", &self.path, error))?;
        replacement
            .write_all(bytes)
            .map_err(|error| io_error("write temporary for", &self.path, error))?;
        Ok(PendingReplacement {
            publication: self,
            replacement,
        })
    }
}

/// Fully written temporary image awaiting its atomic commit.
pub(super) struct PendingReplacement<'a> {
    publication: &'a PublicationLock,
    replacement: AtomicWriteFile,
}

impl PendingReplacement<'_> {
    /// Atomically commits the prepared image while its publication lock is held.
    pub(super) fn commit(self) -> Result<(), String> {
        self.replacement
            .commit()
            .map_err(|error| io_error("replace", &self.publication.path, error))
    }
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "filesystem concurrency fixtures use expect for compact assertions"
    )]

    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Barrier, mpsc};
    use std::time::Duration;

    use fpas_program::{Digest, ProgramIdentity, ProgramImage};

    use super::{PublicationLock, replace};

    fn temp_path(name: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("fpas-build-atomic-{}-{id}", std::process::id()))
            .join(name)
    }

    fn image_bytes(marker: u8) -> Vec<u8> {
        let (program, diagnostics) = fpas_parser::parse("program AtomicFixture; begin end.");
        assert!(diagnostics.is_empty());
        let executable = fpas_compiler::compile(&program).expect("fixture must compile");
        let image = ProgramImage::new(
            ProgramIdentity {
                compiler_version: "atomic-test".to_string(),
                bytecode_version: fpas_bytecode::BYTECODE_VERSION,
                source_hash: Digest::of([marker]),
                options_hash: Digest::of(b"options"),
                units: Vec::new(),
            },
            vec!["main.fpas".to_string()],
            vec![Digest::of([marker])],
            executable,
        )
        .expect("valid program image");
        fpas_program::encode(&image).expect("program image encoding")
    }

    #[test]
    fn concurrent_writers_publish_one_complete_image() {
        let path = temp_path("concurrent.fpascp");
        std::fs::create_dir_all(path.parent().expect("temporary parent"))
            .expect("temporary directory");
        let barrier = Arc::new(Barrier::new(5));
        let mut writers = Vec::new();
        for marker in 1..=4 {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            writers.push(std::thread::spawn(move || {
                let bytes = image_bytes(marker);
                barrier.wait();
                replace(&path, &bytes)
            }));
        }
        barrier.wait();
        for writer in writers {
            writer.join().expect("writer thread").expect("publication");
        }

        let published = std::fs::read(&path).expect("published image");
        fpas_program::decode(&published).expect("complete published image");
        assert!((1..=4).any(|marker| published == image_bytes(marker)));
        std::fs::remove_dir_all(path.parent().expect("temporary parent")).ok();
    }

    #[test]
    fn live_writer_held_beyond_ten_seconds_keeps_its_os_lock() {
        let path = temp_path("long-writer.fpascp");
        std::fs::create_dir_all(path.parent().expect("temporary parent"))
            .expect("temporary directory");
        let lock = PublicationLock::acquire(&path).expect("first writer lock");
        let writer_path = path.clone();
        let (finished_tx, finished_rx) = mpsc::channel();
        let writer = std::thread::spawn(move || {
            finished_tx
                .send(replace(&writer_path, &image_bytes(2)))
                .expect("writer result receiver")
        });

        std::thread::sleep(Duration::from_millis(10_100));
        assert!(
            finished_rx.try_recv().is_err(),
            "second writer must still wait"
        );
        drop(lock);
        finished_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("unblocked writer result")
            .expect("publication after lock release");
        writer.join().expect("writer thread");
        fpas_program::decode(&std::fs::read(&path).expect("published image"))
            .expect("complete published image");
        std::fs::remove_dir_all(path.parent().expect("temporary parent")).ok();
    }
}
