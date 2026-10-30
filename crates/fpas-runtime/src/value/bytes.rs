//! Contiguous, fallible copy-on-write storage for `Std.Bytes.ByteBuffer`.

#[cfg(test)]
mod tests;

use std::{fmt, ops::Deref, sync::Arc};

/// Maximum logical length and accessible capacity of a byte buffer.
pub const MAX_BYTE_BUFFER_LENGTH: usize = 256 * 1024 * 1024;

/// Storage failure before committing a byte-buffer mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteStorageError {
    /// Requested length or capacity exceeds the public limit.
    LimitExceeded,
    /// The native allocation of this many data bytes failed.
    AllocationFailed(usize),
    /// A byte range does not exist in its source or destination.
    InvalidRange,
}

/// Shared byte value; clones share storage and every write preserves other owners.
///
/// Backing allocations are not pooled. Their final owner releases the native vector.
#[derive(Clone)]
pub struct SharedBytes(Arc<Vec<u8>>);

impl SharedBytes {
    #[cfg(test)]
    pub(crate) fn fail_next_allocation() {
        tests::FAIL_ALLOCATION.with(|fail| fail.set(true));
    }

    /// Allocate `length` initialized zero bytes with exactly that capacity.
    pub fn zeroed(length: usize) -> Result<Self, ByteStorageError> {
        let mut data = allocate(length)?;
        data.resize(length, 0);
        Ok(Self(Arc::new(data)))
    }

    /// Copy initialized bytes into an independent exact-capacity allocation.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, ByteStorageError> {
        let mut data = allocate(bytes.len())?;
        data.extend_from_slice(bytes);
        Ok(Self(Arc::new(data)))
    }

    /// Adopt native result storage without expanding it into tagged values.
    pub fn from_vec(data: Vec<u8>) -> Result<Self, ByteStorageError> {
        check_length(data.len())?;
        if data.is_empty() {
            return Ok(Self(Arc::new(Vec::new())));
        }
        if data.capacity() > MAX_BYTE_BUFFER_LENGTH {
            return Self::from_slice(&data);
        }
        Ok(Self(Arc::new(data)))
    }

    /// Accessible capacity retained by this value, excluding allocator metadata.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Return a checked independent slice; only a nonempty full slice shares backing.
    pub fn slice(&self, start: usize, length: usize) -> Result<Self, ByteStorageError> {
        let range = checked_range(self.len(), start, length)?;
        if length != 0 && start == 0 && length == self.len() {
            Ok(self.clone())
        } else {
            Self::from_slice(&self[range])
        }
    }

    /// Replace one byte, detaching shared storage only after all checks succeed.
    pub fn set_at(&mut self, index: usize, value: u8) -> Result<(), ByteStorageError> {
        checked_range(self.len(), index, 1)?;
        self.writable(self.len())?[index] = value;
        Ok(())
    }

    /// Fill a checked existing range without changing logical length.
    pub fn fill(&mut self, start: usize, length: usize, value: u8) -> Result<(), ByteStorageError> {
        let range = checked_range(self.len(), start, length)?;
        if length != 0 {
            self.writable(self.len())?[range].fill(value);
        }
        Ok(())
    }

    /// Overwrite a checked range from a stable source value, including overlapping aliases.
    pub fn copy_from(
        &mut self,
        source: &Self,
        source_start: usize,
        target_start: usize,
        length: usize,
    ) -> Result<(), ByteStorageError> {
        let source_range = checked_range(source.len(), source_start, length)?;
        let target_range = checked_range(self.len(), target_start, length)?;
        if length != 0 {
            self.writable(self.len())?[target_range].copy_from_slice(&source[source_range]);
        }
        Ok(())
    }

    /// Append a stable byte value with bounded geometric capacity growth.
    pub fn concat(&mut self, other: &Self) -> Result<(), ByteStorageError> {
        self.extend_from_slice(other)
    }

    /// Append initialized native bytes without creating an intermediate byte-buffer value.
    pub fn extend_from_slice(&mut self, other: &[u8]) -> Result<(), ByteStorageError> {
        if other.len() > MAX_BYTE_BUFFER_LENGTH - self.len() {
            return Err(ByteStorageError::LimitExceeded);
        }
        if !other.is_empty() {
            self.writable(self.len() + other.len())?
                .extend_from_slice(other);
        }
        Ok(())
    }

    /// Change logical length, retaining capacity and initializing new bytes to zero.
    pub fn resize(&mut self, length: usize) -> Result<(), ByteStorageError> {
        check_length(length)?;
        if length != self.len() {
            self.writable(length)?.resize(length, 0);
        }
        Ok(())
    }

    /// Reserve a minimum total capacity without changing length or shrinking storage.
    pub fn reserve(&mut self, capacity: usize) -> Result<(), ByteStorageError> {
        check_length(capacity)?;
        if capacity > self.capacity() {
            self.writable(capacity)?;
        }
        Ok(())
    }

    /// Reduce capacity exactly to logical length, dropping empty backing references.
    pub fn trim_capacity(&mut self) -> Result<(), ByteStorageError> {
        if self.capacity() != self.len() {
            *self = Self::from_slice(self)?;
        }
        Ok(())
    }

    fn writable(&mut self, required: usize) -> Result<&mut Vec<u8>, ByteStorageError> {
        check_length(required)?;
        let capacity = growth(self.capacity(), required);
        if Arc::get_mut(&mut self.0).is_none() {
            let mut data = allocate(capacity)?;
            data.extend_from_slice(self);
            self.0 = Arc::new(data);
        }
        let Some(data) = Arc::get_mut(&mut self.0) else {
            unreachable!("fresh or unique byte storage has one owner");
        };
        if capacity > data.capacity() {
            reserve_exact(data, capacity)?;
        }
        Ok(data)
    }
}

impl Deref for SharedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for SharedBytes {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.as_ref() == other.as_ref()
    }
}

impl Eq for SharedBytes {}

impl AsRef<[u8]> for SharedBytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl fmt::Display for SharedBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ByteBuffer(Length := {}, Hex := '", self.len())?;
        for byte in self.iter().take(32) {
            write!(formatter, "{byte:02X}")?;
        }
        if self.len() > 32 {
            formatter.write_str("...")?;
        }
        formatter.write_str("')")
    }
}

impl fmt::Debug for SharedBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

fn check_length(length: usize) -> Result<(), ByteStorageError> {
    if length > MAX_BYTE_BUFFER_LENGTH {
        Err(ByteStorageError::LimitExceeded)
    } else {
        Ok(())
    }
}

fn checked_range(
    total: usize,
    start: usize,
    length: usize,
) -> Result<std::ops::Range<usize>, ByteStorageError> {
    if start > total || length > total - start {
        Err(ByteStorageError::InvalidRange)
    } else {
        Ok(start..start + length)
    }
}

fn growth(current: usize, required: usize) -> usize {
    if required <= current {
        return current;
    }
    let mut capacity = current.max(64);
    while capacity < required {
        capacity = (capacity * 2).min(MAX_BYTE_BUFFER_LENGTH);
    }
    capacity
}

fn allocate(capacity: usize) -> Result<Vec<u8>, ByteStorageError> {
    check_length(capacity)?;
    let mut data = Vec::new();
    reserve_exact(&mut data, capacity)?;
    Ok(data)
}

fn reserve_exact(data: &mut Vec<u8>, capacity: usize) -> Result<(), ByteStorageError> {
    #[cfg(test)]
    if capacity != 0 && tests::fail_allocation() {
        return Err(ByteStorageError::AllocationFailed(capacity));
    }
    data.try_reserve_exact(capacity - data.len())
        .map_err(|_| ByteStorageError::AllocationFailed(capacity))
}
