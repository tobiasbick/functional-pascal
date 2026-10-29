//! Serialized file operations validate before observing cooperative cancellation.

use super::failure::Failure;
use super::registry::Files;
use super::state::{FileState, MAX_BYTES};

impl Files {
    fn with<T>(
        &self,
        handle: u64,
        read: bool,
        write: bool,
        mut cancelled: impl FnMut() -> bool,
        action: impl FnOnce(&mut FileState, &str, &mut dyn FnMut() -> bool) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let entry = self.entry(handle)?;
        let mut state = entry
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let state = state.as_mut().ok_or(Failure::Closed)?;
        if read && !entry.readable {
            return Err(Failure::argument("File", "File was not opened for reading"));
        }
        if write && !entry.writable {
            return Err(Failure::argument("File", "File was not opened for writing"));
        }
        if cancelled() {
            return Err(Failure::Cancelled);
        }
        action(state, &entry.path, &mut cancelled)
    }

    pub(super) fn read(
        &self,
        handle: u64,
        size: i64,
        cancelled: impl FnMut() -> bool,
    ) -> Result<Vec<u8>, Failure> {
        if size <= 0 {
            return Err(Failure::argument("MaxBytes", "Read size must be positive"));
        }
        if size > MAX_BYTES as i64 {
            return Err(Failure::Limit(MAX_BYTES as i64));
        }
        self.with(handle, true, false, cancelled, |state, path, _| {
            state.read(size as usize, path)
        })
    }

    pub(super) fn write(
        &self,
        handle: u64,
        bytes: &[u8],
        cancelled: impl FnMut() -> bool,
    ) -> Result<usize, Failure> {
        if bytes.len() > MAX_BYTES {
            return Err(Failure::Limit(MAX_BYTES as i64));
        }
        self.with(handle, false, true, cancelled, |state, path, _| {
            state.write(bytes, path)
        })
    }

    pub(super) fn read_line(
        &self,
        handle: u64,
        cancelled: impl FnMut() -> bool,
    ) -> Result<Option<String>, Failure> {
        self.with(handle, true, false, cancelled, |state, path, cancelled| {
            state.read_line(path, cancelled)
        })
    }

    pub(super) fn size(&self, handle: u64) -> Result<i64, Failure> {
        self.with(
            handle,
            false,
            false,
            || false,
            |state, path, _| state.size(path),
        )
    }

    pub(super) fn set_length(&self, handle: u64, length: i64) -> Result<(), Failure> {
        if length < 0 {
            return Err(Failure::argument(
                "Length",
                "File length must not be negative",
            ));
        }
        self.with(
            handle,
            false,
            true,
            || false,
            |state, path, _| state.set_length(length as u64, path),
        )
    }

    pub(super) fn write_text(
        &self,
        handle: u64,
        text: &str,
        newline: bool,
        cancelled: impl FnMut() -> bool,
    ) -> Result<(), Failure> {
        if text.len() > MAX_BYTES {
            return Err(Failure::Limit(MAX_BYTES as i64));
        }
        self.with(handle, false, true, cancelled, |state, path, cancelled| {
            state.write_text(text, newline, path, cancelled)
        })
    }

    pub(super) fn seek(&self, handle: u64, offset: i64, origin: i64) -> Result<i64, Failure> {
        if !(0..=2).contains(&origin) {
            return Err(Failure::argument("Origin", "Unknown seek origin"));
        }
        if origin == 0 && offset < 0 {
            return Err(Failure::argument(
                "Offset",
                "Start offset must not be negative",
            ));
        }
        self.with(
            handle,
            false,
            false,
            || false,
            |state, path, _| state.seek(offset, origin, path),
        )
    }

    pub(super) fn tell(&self, handle: u64) -> Result<i64, Failure> {
        self.with(
            handle,
            false,
            false,
            || false,
            |state, path, _| state.tell(path),
        )
    }

    pub(super) fn flush(&self, handle: u64) -> Result<(), Failure> {
        self.with(
            handle,
            false,
            false,
            || false,
            |state, path, _| state.flush(path),
        )
    }
}
