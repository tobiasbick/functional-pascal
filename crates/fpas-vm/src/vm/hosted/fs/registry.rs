//! VM-owned regular files with admission before any destructive operation.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Seek, SeekFrom};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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

pub(in crate::vm::hosted) struct Files {
    entries: Mutex<Registry>,
}

impl Files {
    pub(in crate::vm::hosted) fn new() -> Self {
        Self {
            entries: Mutex::new(Registry {
                files: HashMap::new(),
                shutdown: false,
                limit: MAX_FILES,
            }),
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
        }
    }

    pub(super) fn open(&self, path: &str, mode: i64) -> Result<u64, Failure> {
        if !(0..=3).contains(&mode) {
            return Err(Failure::argument("Mode", "Unknown file mode"));
        }
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if entries.shutdown {
            return Err(Failure::Closed);
        }
        if entries.files.len() >= entries.limit {
            return Err(Failure::Limit(entries.limit as i64));
        }
        if path.contains('\0') {
            return Err(Failure::argument("Path", "Path must not contain NUL"));
        }
        // Holding admission across open also prevents cap races with Create truncation.
        let handle = NEXT_HANDLE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
                    .filter(|next| next & HANDLE_MASK == HANDLE_TAG)
            })
            .map_err(|_| Failure::Limit(MAX_FILES as i64))?;
        let file = open_regular(path, mode)?;
        entries.files.insert(
            handle,
            Arc::new(Entry {
                path: path.to_owned(),
                readable: mode == 0 || mode == 3,
                writable: mode != 0,
                state: Mutex::new(Some(FileState::new(BufReader::new(file)))),
            }),
        );
        Ok(handle)
    }

    pub(super) fn entry(&self, handle: u64) -> Result<Arc<Entry>, Failure> {
        validate_handle(handle)?;
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .files
            .get(&handle)
            .cloned()
            .ok_or(Failure::Closed)
    }

    pub(super) fn close(&self, handle: u64) -> Result<(), Failure> {
        validate_handle(handle)?;
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let entry = entries.files.remove(&handle).ok_or(Failure::Closed)?;
        // Keep admission locked until the actual OS descriptor has been released.
        drop(
            entry
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .take(),
        );
        Ok(())
    }

    pub(in crate::vm::hosted) fn shutdown(&self) {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        entries.shutdown = true;
        for (_, entry) in entries.files.drain() {
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
