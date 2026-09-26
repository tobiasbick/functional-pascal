//! Little-endian field writers.

use crate::Digest;

/// Append one byte.
#[inline]
pub fn write_u8(output: &mut Vec<u8>, value: u8) {
    output.push(value);
}

/// Append a little-endian `u16`.
#[inline]
pub fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

/// Append a little-endian `u32`.
#[inline]
pub fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

/// Append a little-endian `u64`.
#[inline]
pub fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

/// Append a little-endian `i64`.
#[inline]
pub fn write_i64(output: &mut Vec<u8>, value: i64) {
    output.extend_from_slice(&value.to_le_bytes());
}

/// Append the raw digest bytes.
#[inline]
pub fn write_digest(output: &mut Vec<u8>, digest: Digest) {
    output.extend_from_slice(digest.as_bytes());
}
