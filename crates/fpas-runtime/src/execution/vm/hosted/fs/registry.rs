//! VM-owned regular files with admission before any destructive operation.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use super::failure::Failure;
use super::state::FileState;

const HANDLE_TAG: u64 = 0x4653_0000_0000_0000;
const HANDLE_MASK: u64 = 0xFFFF_0000_0000_0000;
const MAX_FILES: usize = 1024;
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(HANDLE_TAG | 1);

pub(super) struct Entry {
    pub path: String,
    pub readable: bool,
    pub writable: bool,
    pub state: Mutex<Option<FileState>>,
}

struct Registry {
    files: HashMap<u64, Arc<Entry>>,
    shutdown: bool,
    limit: usize,
}

pub(in crate::execution::vm::hosted) struct Files {
    entries: Mutex<Registry>,
    stopping: AtomicBool,
}

impl Files {
    pub(in crate::execution::vm::hosted) fn new() -> Self {
        Self {
            entries: Mutex::new(Registry {
                files: HashMap::new(),
                shutdown: false,
                limit: MAX_FILES,
            }),
            stopping: AtomicBool::new(false),
        }
    }

    #[cfg(test)]
    pub(super) fn with_limit(limit: usize) -> Self {
        Self {
            entries: Mutex::new(Registry {
                files: HashMap::new(),
                shutdown: false,
                limit,
            }),
            stopping: AtomicBool::new(false),
        }
    }

    pub(super) fn open(
        &self,
        path: &str,
        mode: i64,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<u64, Failure> {
        if !(0..=3).contains(&mode) {
            return Err(Failure::argument("Mode", "Unknown file mode"));
        }
        let mut entries = lock(&self.entries, &mut cancelled)?;
        if entries.shutdown || self.is_stopping() {
            return Err(Failure::Closed);
        }
        if entries.files.len() >= entries.limit {
            return Err(Failure::Limit(entries.limit as i64));
        }
        if path.contains('\0') {
            return Err(Failure::argument("Path", "Path must not contain NUL"));
        }
        if cancelled() {
            return Err(Failure::Cancelled);
        }
        // Holding admission across open also prevents cap races with Create truncation.
        let handle = NEXT_HANDLE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
                    .filter(|next| next & HANDLE_MASK == HANDLE_TAG)
            })
            .map_err(|_| Failure::Limit(MAX_FILES as i64))?;
        let file = open_regular(path, mode)?;
        if self.is_stopping() || cancelled() {
            return Err(Failure::Cancelled);
        }
        entries.files.insert(
            handle,
            Arc::new(Entry {
                path: path.to_owned(),
                readable: mode == 0 || mode == 3,
                writable: mode != 0,
                state: Mutex::new(Some(FileState::new(BufReader::new(file), mode == 2))),
            }),
        );
        Ok(handle)
    }

    pub(super) fn entry(
        &self,
        handle: u64,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<Arc<Entry>, Failure> {
        validate_handle(handle)?;
        lock(&self.entries, cancelled)?
            .files
            .get(&handle)
            .cloned()
            .ok_or(Failure::Closed)
    }

    pub(super) fn close(
        &self,
        handle: u64,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<(), Failure> {
        let entry = self.entry(handle, &mut cancelled)?;
        let mut state = lock(&entry.state, &mut cancelled)?;
        if state.is_none() {
            return Err(Failure::Closed);
        }
        if cancelled() {
            return Err(Failure::Cancelled);
        }
        // Keep the slot admitted until its OS descriptor is actually released.
        drop(state.take());
        drop(state);
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .files
            .remove(&handle);
        Ok(())
    }

    pub(super) fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(super) fn handles(&self) -> Vec<u64> {
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .files
            .keys()
            .copied()
            .collect()
    }

    /// Signal active jobs without waiting for a filesystem syscall or alias lock.
    pub(in crate::execution::vm::hosted) fn request_shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
    }

    pub(in crate::execution::vm::hosted) fn shutdown(&self) {
        self.request_shutdown();
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        entries.shutdown = true;
        let files = std::mem::take(&mut entries.files);
        drop(entries);
        for entry in files.into_values() {
            drop(
                entry
                    .state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .take(),
            );
        }
    }
}

/// Observe cancellation while waiting behind another operation, never while owning its lock.
pub(super) fn lock<'a, T>(
    mutex: &'a Mutex<T>,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<MutexGuard<'a, T>, Failure> {
    loop {
        match mutex.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(error)) => return Ok(error.into_inner()),
            Err(TryLockError::WouldBlock) => {
                if cancelled() {
                    return Err(Failure::Cancelled);
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn validate_handle(handle: u64) -> Result<(), Failure> {
    if handle & HANDLE_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err(Failure::argument("File", "Value is not a file handle"))
    }
}

fn open_regular(path: &str, mode: i64) -> Result<File, Failure> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(Failure::argument("Path", "Path must name a regular file"));
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(Failure::io(path, error));
        }
        _ => {}
    }
    let mut options = OpenOptions::new();
    options
        .read(mode == 0 || mode == 3)
        .write(mode != 0)
        .create(mode == 1 || mode == 2)
        .truncate(false)
        .append(mode == 2);
    #[cfg(windows)]
    if mode == 2 {
        use std::os::windows::fs::OpenOptionsExt;
        // GENERIC_WRITE permits resizing. Append writes use the explicit EOF
        // offset instead of relying on an append-only access mask.
        options.access_mode(0x4000_0000);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let mut file = options
        .open(path)
        .map_err(|error| Failure::io(path, error))?;
    if !file
        .metadata()
        .map_err(|error| Failure::io(path, error))?
        .is_file()
    {
        return Err(Failure::argument("Path", "Path must name a regular file"));
    }
    if mode == 1 {
        file.set_len(0).map_err(|error| Failure::io(path, error))?;
    }
    if mode == 2 {
        file.seek(SeekFrom::End(0))
            .map_err(|error| Failure::io(path, error))?;
    }
    Ok(file)
}
