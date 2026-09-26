//! Bounds-checked little-endian field reader.

use std::marker::PhantomData;

use crate::Digest;

/// Decoding failure raised by [`ByteReader`].
///
/// Format crates convert it into their own error type through `From`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// Required bytes are missing.
    Truncated(&'static str),
    /// A length exceeds the accepted maximum.
    LimitExceeded {
        /// Logical field name.
        field: &'static str,
        /// Encoded or requested size.
        size: usize,
        /// Largest accepted size.
        maximum: usize,
    },
    /// A string field is not valid UTF-8.
    InvalidUtf8(&'static str),
}

/// Sequential reader over an encoded byte slice.
///
/// `E` is the caller's format error; every failure is converted from [`ReadError`].
pub struct ByteReader<'a, E> {
    bytes: &'a [u8],
    position: usize,
    error: PhantomData<fn() -> E>,
}

impl<'a, E: From<ReadError>> ByteReader<'a, E> {
    /// Start reading at the first byte of `bytes`.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            error: PhantomData,
        }
    }

    /// Number of unread bytes.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    /// Read exactly `length` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when fewer bytes remain, or
    /// [`ReadError::LimitExceeded`] when `length` overflows the position.
    #[inline]
    pub fn take(&mut self, length: usize, field: &'static str) -> Result<&'a [u8], E> {
        let Some(end) = self.position.checked_add(length) else {
            return Err(ReadError::LimitExceeded {
                field,
                size: length,
                maximum: self.remaining(),
            }
            .into());
        };
        let Some(value) = self.bytes.get(self.position..end) else {
            return Err(ReadError::Truncated(field).into());
        };
        self.position = end;
        Ok(value)
    }

    /// Read exactly `N` bytes into an array.
    #[inline]
    fn array<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N], E> {
        let mut value = [0_u8; N];
        value.copy_from_slice(self.take(N, field)?);
        Ok(value)
    }

    /// Read one byte.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when no byte remains.
    #[inline]
    pub fn u8(&mut self, field: &'static str) -> Result<u8, E> {
        Ok(self.take(1, field)?[0])
    }

    /// Read a little-endian `u16`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when fewer than two bytes remain.
    #[inline]
    pub fn u16(&mut self, field: &'static str) -> Result<u16, E> {
        self.array(field).map(u16::from_le_bytes)
    }

    /// Read a little-endian `u32`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when fewer than four bytes remain.
    #[inline]
    pub fn u32(&mut self, field: &'static str) -> Result<u32, E> {
        self.array(field).map(u32::from_le_bytes)
    }

    /// Read a little-endian `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when fewer than eight bytes remain.
    #[inline]
    pub fn u64(&mut self, field: &'static str) -> Result<u64, E> {
        self.array(field).map(u64::from_le_bytes)
    }

    /// Read a little-endian `i64`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when fewer than eight bytes remain.
    #[inline]
    pub fn i64(&mut self, field: &'static str) -> Result<i64, E> {
        self.array(field).map(i64::from_le_bytes)
    }

    /// Read a raw [`Digest`].
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::Truncated`] when fewer than [`Digest::LENGTH`] bytes remain.
    pub fn digest(&mut self, field: &'static str) -> Result<Digest, E> {
        self.array(field).map(Digest::from_bytes)
    }

    /// Read a `u32`-length-prefixed payload of at most `maximum` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ReadError::LimitExceeded`] above `maximum` or
    /// [`ReadError::Truncated`] when the payload is incomplete.
    pub fn bytes(&mut self, field: &'static str, maximum: usize) -> Result<&'a [u8], E> {
        let length = self.u32(field)? as usize;
        if length > maximum {
            return Err(ReadError::LimitExceeded {
                field,
                size: length,
                maximum,
            }
            .into());
        }
        self.take(length, field)
    }

    /// Read a `u32`-length-prefixed UTF-8 string of at most `maximum` bytes.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::bytes`] or [`ReadError::InvalidUtf8`].
    pub fn string(&mut self, field: &'static str, maximum: usize) -> Result<String, E> {
        let bytes = self.bytes(field, maximum)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| ReadError::InvalidUtf8(field).into())
    }
}
